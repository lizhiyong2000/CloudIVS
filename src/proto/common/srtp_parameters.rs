use strum;
use strum_macros::EnumString;

/**
 * SRTP crypto suite.
 */
#[derive(PartialEq, Debug, EnumString)]
enum SrtpCryptoSuite {
    #[strum(serialize = "AES_CM_128_HMAC_SHA1_80")]
    AES_CM_128_HMAC_SHA1_80,

    #[strum(serialize = "AES_CM_128_HMAC_SHA1_32")]
    AES_CM_128_HMAC_SHA1_32,
}

/**
 * SRTP parameters.
 */
struct SrtpParameters {
    /**
     * Encryption and authentication transforms to be used.
     */
    cryptoSuite: SrtpCryptoSuite,

    /**
     * SRTP keying material (master key and salt) in Base64.
     */
    keyBase64: String,
}



