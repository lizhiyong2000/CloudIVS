use std::collections::HashMap;
use std::str::FromStr;

use strum;
use strum_macros::EnumString;

/**
 * Media kind ('audio' or 'video').
 */
#[derive(PartialEq, Display, Debug, EnumString)]
enum MediaKind {
    #[strum(serialize = "audio", serialize = "a")]
    Audio,

    #[strum(serialize = "video", serialize = "v")]
    Video,
}


/**
 * Direction of RTP header extension.
 */
#[derive(PartialEq, Display, Debug, EnumString)]
enum RtpHeaderExtensionDirection {
    #[strum(serialize = "sendrecv", serialize = "sr")]
    SendRecv,

    #[strum(serialize = "sendonly", serialize = "s")]
    SendOnly,

    #[strum(serialize = "recvonly", serialize = "r")]
    RecvOnly,

    #[strum(serialize = "inactive", serialize = "i")]
    Inactive,
}


struct RtpCodecCapability
{
    /**
     * Media kind.
     */
    kind: MediaKind,

    /**
     * The codec MIME media type/subtype (e.g. 'audio/opus', 'video/VP8').
     */
    mimeType: String,

    /**
     * The preferred RTP payload type.
     */
    preferredPayloadType: u32,

    /**
     * Codec clock rate expressed in Hertz.
     */
    clockRate: u64,

    /**
     * The number of channels supported (e.g. two for stereo). Just for audio.
     * Default 1.
     */
    channels: u8,

    /**
     * Codec specific parameters. Some parameters (such as 'packetization-mode'
     * and 'profile-level-id' in H264 or 'profile-id' in VP9) are critical for
     * codec matching.
     */
    parameters: HashMap<String, String>,

    /**
     * Transport layer and codec-specific feedback messages for this codec.
     */
    rtcpFeedback: Vec<RtcpFeedback>,
}

/**
 * Provides information relating to supported header extensions. The list of
 * RTP header extensions supported by mediasoup is defined in the
 * supportedRtpCapabilities.ts file.
 *
 * mediasoup does not currently support encrypted RTP header extensions. The
 * direction field is just present in mediasoup RTP capabilities (retrieved via
 * router.rtpCapabilities or mediasoup.getSupportedRtpCapabilities()). It's
 * ignored if present in endpoints' RTP capabilities.
 */
struct RtpHeaderExtension {
    /**
     * Media kind. If empty string, it's valid for all kinds.
     * Default any media kind.
     */
    kind: Option<MediaKind>,

    /*
     * The URI of the RTP header extension, as defined in RFC 5285.
     */
    uri: String,

    /**
     * The preferred numeric identifier that goes in the RTP packet. Must be
     * unique.
     */
    preferredId: u64,

    /**
     * If true, it is preferred that the value in the header be encrypted as per
     * RFC 6904. Default false.
     */
    preferredEncrypt: bool,

    /**
     * If 'sendrecv', mediasoup supports sending and receiving this RTP extension.
     * 'sendonly' means that mediasoup can send (but not receive) it. 'recvonly'
     * means that mediasoup can receive (but not send) it.
     */
    direction: RtpHeaderExtensionDirection,
}


/**
 * The RTP capabilities define what mediasoup or an endpoint can receive at
 * media level.
 */
struct RtpCapabilities {
    /**
     * Supported media and RTX codecs.
     */
    codecs: Vec<RtpCodecCapability>,

    /**
     * Supported RTP header extensions.
     */
    headerExtensions: Vec<RtpHeaderExtension>,

    /**
     * Supported FEC mechanisms.
     */
    fecMechanisms: Vec<String>,
}


/**
 * Provides information on codec settings within the RTP parameters. The list
 * of media codecs supported by mediasoup and their settings is defined in the
 * supportedRtpCapabilities.ts file.
 */
struct RtpCodecParameters {
    /**
     * The codec MIME media type/subtype (e.g. 'audio/opus', 'video/VP8').
     */
    mimeType: String,

    /**
     * The value that goes in the RTP Payload Type Field. Must be unique.
     */
    payloadType: u32,

    /**
     * Codec clock rate expressed in Hertz.
     */
    clockRate: u64,

    /**
     * The number of channels supported (e.g. two for stereo). Just for audio.
     * Default 1.
     */
    channels: u8,

    /**
     * Codec-specific parameters available for signaling. Some parameters (such
     * as 'packetization-mode' and 'profile-level-id' in H264 or 'profile-id' in
     * VP9) are critical for codec matching.
     */
    parameters: HashMap<String, String>,

    /**
     * Transport layer and codec-specific feedback messages for this codec.
     */
    rtcpFeedback: Vec<RtcpFeedback>,
}

/**
 * Provides information on RTCP feedback messages for a specific codec. Those
 * messages can be transport layer feedback messages or codec-specific feedback
 * messages. The list of RTCP feedbacks supported by mediasoup is defined in the
 * supportedRtpCapabilities.ts file.
 */
struct RtcpFeedback {
    /**
     * RTCP feedback type.
     */
    feedback_type: String,

    /**
     * RTCP feedback parameter.
     */
    parameter: String,
}

/**
 * Provides information relating to an encoding, which represents a media RTP
 * stream and its associated RTX stream (if any).
 */
struct RtpEncodingParameters {
    /**
     * The media SSRC.
     */
    ssrc: u64,

    /**
     * The RID RTP extension value. Must be unique.
     */
    rid: String,

    /**
     * Codec payload type this encoding affects. If unset, first media codec is
     * chosen.
     */
    codecPayloadType: u32,

    /**
     * RTX stream information. It must contain a numeric ssrc field indicating
     * the RTX SSRC.
     */
    rtx_ssrc: u64,

    /**
     * It indicates whether discontinuous RTP transmission will be used. Useful
     * for audio (if the codec supports it) and for video screen sharing (when
     * static content is being transmitted, this option disables the RTP
     * inactivity checks in mediasoup). Default false.
     */
    dtx: bool,

    /**
     * Number of spatial and temporal layers in the RTP stream (e.g. 'L1T3').
     * See webrtc-svc.
     */
    scalabilityMode: String,

    /**
     * Others.
     */
    scaleResolutionDownBy: u64,
    maxBitrate: u64,
}

/**
 * Defines a RTP header extension within the RTP parameters. The list of RTP
 * header extensions supported by mediasoup is defined in the
 * supportedRtpCapabilities.ts file.
 *
 * mediasoup does not currently support encrypted RTP header extensions and no
 * parameters are currently considered.
 */
struct RtpHeaderExtensionParameters {
    /**
     * The URI of the RTP header extension, as defined in RFC 5285.
     */
    uri: String,

    /**
     * The numeric identifier that goes in the RTP packet. Must be unique.
     */
    id: u64,

    /**
     * If true, the value in the header is encrypted as per RFC 6904. Default false.
     */
    encrypt: bool,

    /**
     * Configuration parameters for the header extension.
     */
    parameters: HashMap<String, String>,
}

/**
 * Provides information on RTCP settings within the RTP parameters.
 *
 * If no cname is given in a producer's RTP parameters, the mediasoup transport
 * will choose a random one that will be used into RTCP SDES messages sent to
 * all its associated consumers.
 *
 * mediasoup assumes reducedSize to always be true.
 */
struct RtcpParameters {
    /**
     * The Canonical Name (CNAME) used by RTCP (e.g. in SDES messages).
     */
    cname: String,

    /**
     * Whether reduced size RTCP RFC 5506 is configured (if true) or compound RTCP
     * as specified in RFC 3550 (if false). Default true.
     */
    reducedSize: bool,

    /**
     * Whether RTCP-mux is used. Default true.
     */
    mux: bool,
}


/**
 * The RTP send parameters describe a media stream received by mediasoup from
 * an endpoint through its corresponding mediasoup Producer. These parameters
 * may include a mid value that the mediasoup transport will use to match
 * received RTP packets based on their MID RTP extension value.
 *
 * mediasoup allows RTP send parameters with a single encoding and with multiple
 * encodings (simulcast). In the latter case, each entry in the encodings array
 * must include a ssrc field or a rid field (the RID RTP extension value). Check
 * the Simulcast and SVC sections for more information.
 *
 * The RTP receive parameters describe a media stream as sent by mediasoup to
 * an endpoint through its corresponding mediasoup Consumer. The mid value is
 * unset (mediasoup does not include the MID RTP extension into RTP packets
 * being sent to endpoints).
 *
 * There is a single entry in the encodings array (even if the corresponding
 * producer uses simulcast). The consumer sends a single and continuous RTP
 * stream to the endpoint and spatial/temporal layer selection is possible via
 * consumer.setPreferredLayers().
 *
 * As an exception, previous bullet is not true when consuming a stream over a
 * PipeTransport, in which all RTP streams from the associated producer are
 * forwarded verbatim through the consumer.
 *
 * The RTP receive parameters will always have their ssrc values randomly
 * generated for all of its  encodings (and optional rtx: { ssrc: XXXX } if the
 * endpoint supports RTX), regardless of the original RTP send parameters in
 * the associated producer. This applies even if the producer's encodings have
 * rid set.
 */
struct RtpParameters {
    /**
     * The MID RTP extension value as defined in the BUNDLE specification.
     */
    mid: String,
    /**
     * Media and RTX codecs in use.
     */
    codecs: Vec<RtpCodecParameters>,
    /**
     * RTP header extensions in use.
     */
    headerExtensions: Vec<RtpHeaderExtensionParameters>,
    /**
     * Transmitted RTP streams and their settings.
     */
    encodings: Vec<RtpEncodingParameters>,
    /**
     * Parameters used for RTCP.
     */
    rtcp: RtcpParameters,

}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::MediaKind;

    #[test]
    fn test_media_kind() {
        let video_kind = MediaKind::from_str("video").unwrap();
        let audio_kind = MediaKind::from_str("a").unwrap();

        // assert!(std::matches!(MediaKind::Video, video_kind));
        assert_matches!(MediaKind::Audio, audio_kind);

        assert_ne!(audio_kind, video_kind);
    }
}



