use tokio_util::codec::{Decoder, Encoder};

use tokio::{net::UdpSocket, stream::Stream};

use bytes::{BufMut, BytesMut};
// use futures_core::ready;
// use futures_sink::Sink;
use std::io;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Sink;
use futures::ready;
use std::ops::Deref;

use log::info;

/// A unified `Stream` and `Sink` interface to an underlying `UdpSocket`, using
/// the `Encoder` and `Decoder` traits to encode and decode frames.
///
/// Raw UDP sockets work with datagrams, but higher-level code usually wants to
/// batch these into meaningful chunks, called "frames". This method layers
/// framing on top of this socket by using the `Encoder` and `Decoder` traits to
/// handle encoding and decoding of messages frames. Note that the incoming and
/// outgoing frame types may be distinct.
///
/// This function returns a *single* object that is both `Stream` and `Sink`;
/// grouping this into a single object is often useful for layering things which
/// require both read and write access to the underlying object.
///
/// If you want to work more directly with the streams and sink, consider
/// calling `split` on the `UdpFramed` returned by this method, which will break
/// them into separate objects, allowing them to interact more easily.
#[must_use = "sinks do nothing unless polled"]
#[cfg_attr(docsrs, doc(all(feature = "codec", feature = "udp")))]
#[derive(Debug)]
pub struct UdpFramed<C> {
    socket: UdpSocket,
    codec: C,
    rd: BytesMut,
    wr: BytesMut,
    out_addr: SocketAddr,
    flushed: bool,
    is_readable: bool,
    current_addr: Option<SocketAddr>,
}

impl<C: Decoder + Unpin> Stream for UdpFramed<C> {
    type Item = Result<(C::Item, SocketAddr), C::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pin = self.get_mut();

        // info!("poll_next loop pin.is_readable:{}", pin.is_readable);
        // pin.rd.reserve(INITIAL_RD_CAPACITY);

        loop {
            pin.rd.reserve(INITIAL_RD_CAPACITY);
            // Are there are still bytes left in the read buffer to decode?
            if pin.is_readable {
                if let Some(frame) = pin.codec.decode(&mut pin.rd)? {
                    let current_addr = pin
                        .current_addr
                        .expect("will always be set before this line is called");

                    pin.is_readable = false;
                    pin.rd.clear();

                    // info!("poll_next loop return Poll::Ready(Some(Ok((frame, current_addr))))");
                    return Poll::Ready(Some(Ok((frame, current_addr))));
                    //
                    // return Poll::Pending;
                }

                // if this line has been reached then decode has returned `None`.
                pin.is_readable = false;
                pin.rd.clear();
            }

            // info!("poll_next loop pin.is_readable2:{}", pin.is_readable);

            // We're out of data. Try and fetch more data to decode
            let addr = unsafe {
                // Convert `&mut [MaybeUnit<u8>]` to `&mut [u8]` because we will be
                // writing to it via `poll_recv_from` and therefore initializing the memory.
                let buf: &mut [u8] =
                    &mut *(pin.rd.bytes_mut() as *mut [MaybeUninit<u8>] as *mut [u8]);
                //
                // let res = ready!(Pin::new(&mut pin.socket).poll(cx, buf));
                //
                // let (n, addr) = res?;
                // pin.rd.advance_mut(n);
                // addr
                // info!("futures::executor::block_on");

                let result = pin.socket.recv_from(buf);
                let result = futures::executor::block_on(result);

                // info!("futures::executor::block_on end");

                match result{
                    Ok(res)=>{
                        let (n, addr) = res;
                        pin.rd.advance_mut(n);
                        addr
                    }
                    _ => {
                        info!("poll_next loop return Poll::Ready(None)");
                        return Poll::Ready(None);
                    }
                }
            };

            pin.current_addr = Some(addr);
            pin.is_readable = true;
        }
    }
}

impl<I, C: Encoder<I> + Unpin> Sink<(I, SocketAddr)> for UdpFramed<C> {
    type Error = C::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.flushed {
            match self.poll_flush(cx)? {
                Poll::Ready(()) => {}
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: (I, SocketAddr)) -> Result<(), Self::Error> {
        let (frame, out_addr) = item;

        let pin = self.get_mut();

        pin.codec.encode(frame, &mut pin.wr)?;
        pin.out_addr = out_addr;
        pin.flushed = false;

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.flushed {
            return Poll::Ready(Ok(()));
        }

        let Self {
            ref mut socket,
            ref mut out_addr,
            ref mut wr,
            ..
        } = *self;

        let result = socket.send_to(&wr, out_addr.deref());
        let result = futures::executor::block_on(result);

        match result{
            Ok(res)=>{
                // let (n, addr) = res;
                // pin.rd.advance_mut(n);
                // addr
                let wrote_all = res == self.wr.len();
                self.wr.clear();
                self.flushed = true;

                if wrote_all{
                    return Poll::Ready(Ok(()));
                }
                else {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other,"failed to write entire datagram to socket").into()));
                }
            }
            _ => {
                // return Poll::Ready(None);
                return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::Other,
                        "failed to write entire datagram to socket",
                    )
                        .into()));
            }
        }


        // let n = ready!(socket.poll_send_to(cx, &wr, &out_addr))?;
        //
        // let wrote_all = n == self.wr.len();
        // self.wr.clear();
        // self.flushed = true;
        //
        // let res = if wrote_all {
        //     Ok(())
        // } else {
        //     Err(io::Error::new(
        //         io::ErrorKind::Other,
        //         "failed to write entire datagram to socket",
        //     )
        //         .into())
        // };

        // Poll::Ready(res)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }
}

const INITIAL_RD_CAPACITY: usize = 64 * 1024;
const INITIAL_WR_CAPACITY: usize = 8 * 1024;

impl<C> UdpFramed<C> {
    /// Create a new `UdpFramed` backed by the given socket and codec.
    ///
    /// See struct level documentation for more details.
    pub fn new(socket: UdpSocket, codec: C) -> UdpFramed<C> {
        UdpFramed {
            socket,
            codec,
            out_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
            rd: BytesMut::with_capacity(INITIAL_RD_CAPACITY),
            wr: BytesMut::with_capacity(INITIAL_WR_CAPACITY),
            flushed: true,
            is_readable: false,
            current_addr: None,
        }
    }

    /// Returns a reference to the underlying I/O stream wrapped by `Framed`.
    ///
    /// # Note
    ///
    /// Care should be taken to not tamper with the underlying stream of data
    /// coming in as it may corrupt the stream of frames otherwise being worked
    /// with.
    pub fn get_ref(&self) -> &UdpSocket {
        &self.socket
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `Framed`.
    ///
    /// # Note
    ///
    /// Care should be taken to not tamper with the underlying stream of data
    /// coming in as it may corrupt the stream of frames otherwise being worked
    /// with.
    pub fn get_mut(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }

    /// Consumes the `Framed`, returning its underlying I/O stream.
    pub fn into_inner(self) -> UdpSocket {
        self.socket
    }
}
