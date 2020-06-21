use failure::{Backtrace, Context, Fail};
use std::fmt;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }

    pub fn compat(self) -> failure::Compat<Context<ErrorKind>> {
        self.inner.compat()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

/*impl From<failure::Error> for Error {
    fn from(e: failure::Error) -> Error {
        Error { inner: e.context(ErrorKind::Unknown) }
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for Error {
    fn from(e: E) -> Error {
        let f = e as Fail;
        Error { inner: f.context(ErrorKind::Unknown) }
    }
}*/

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner.cause() {
            None => fmt::Display::fmt(&self.kind(), f),
            Some(c) => write!(f, "{}: {}", self.kind(), c),
        }
    }
}

/// Error kind.
///
/// These codes are taken from
/// [grpc::StatusCode](https://github.com/grpc/grpc/blob/0e00c430827e81d61e1e7164ef04ca21ccbfaa77/include/grpcpp/impl/codegen/status_code_enum.h),
/// which is a nice general-purpose classification of errors. See that link for descriptions of
/// each error.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Cancelled")] Cancelled,
    #[fail(display = "Unknown")] Unknown,
    #[fail(display = "Invalid argument")] InvalidArgument,
    #[fail(display = "Deadline exceeded")] DeadlineExceeded,
    #[fail(display = "Not found")] NotFound,
    #[fail(display = "Already exists")] AlreadyExists,
    #[fail(display = "Permission denied")] PermissionDenied,
    #[fail(display = "Unauthenticated")] Unauthenticated,
    #[fail(display = "Resource exhausted")] ResourceExhausted,
    #[fail(display = "Failed precondition")] FailedPrecondition,
    #[fail(display = "Aborted")] Aborted,
    #[fail(display = "Out of range")] OutOfRange,
    #[fail(display = "Unimplemented")] Unimplemented,
    #[fail(display = "Internal")] Internal,
    #[fail(display = "Unavailable")] Unavailable,
    #[fail(display = "Data loss")] DataLoss,
    #[doc(hidden)] #[fail(display = "__Nonexhaustive")] __Nonexhaustive,
}

/// Extension methods for `Result`.
pub trait ResultExt<T, E> {
    /// Annotates an error with the given kind.
    /// Example:
    /// ```
    /// use base::{ErrorKind, ResultExt};
    /// use std::io::Read;
    /// let mut buf = [0u8; 1];
    /// let r = std::io::Cursor::new("").read_exact(&mut buf[..]).err_kind(ErrorKind::Internal);
    /// assert_eq!(r.unwrap_err().kind(), ErrorKind::Internal);
    /// ```
    fn err_kind(self, k: ErrorKind) -> Result<T, Error>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> where E: Into<failure::Error> {
    fn err_kind(self, k: ErrorKind) -> Result<T, Error> {
        self.map_err(|e| e.into().context(k).into())
    }
}

/// Like `failure::bail!`, but the first argument specifies a type as an `ErrorKind`.
///
/// Example:
/// ```
/// use base::bail_t;
/// let e = || -> Result<(), base::Error> {
///     bail_t!(Unauthenticated, "unknown user: {}", "slamb");
/// }().unwrap_err();
/// assert_eq!(e.kind(), base::ErrorKind::Unauthenticated);
/// assert_eq!(e.to_string(), "Unauthenticated: unknown user: slamb");
/// ```
#[macro_export]
macro_rules! bail_t {
    ($t:ident, $e:expr) => {
        return Err(failure::err_msg($e).context($crate::ErrorKind::$t).into());
    };
    ($t:ident, $fmt:expr, $($arg:tt)+) => {
        return Err(failure::err_msg(format!($fmt, $($arg)+)).context($crate::ErrorKind::$t).into());
    };
}

/// Like `failure::format_err!`, but the first argument specifies a type as an `ErrorKind`.
///
/// Example:
/// ```
/// use base::format_err_t;
/// let e = format_err_t!(Unauthenticated, "unknown user: {}", "slamb");
/// assert_eq!(e.kind(), base::ErrorKind::Unauthenticated);
/// assert_eq!(e.to_string(), "Unauthenticated: unknown user: slamb");
/// ```
#[macro_export]
macro_rules! format_err_t {
    ($t:ident, $e:expr) => {
        Into::<$crate::Error>::into(failure::err_msg($e).context($crate::ErrorKind::$t))
    };
    ($t:ident, $fmt:expr, $($arg:tt)+) => {
        Into::<$crate::Error>::into(failure::err_msg(format!($fmt, $($arg)+))
                                    .context($crate::ErrorKind::$t))
    };
}
