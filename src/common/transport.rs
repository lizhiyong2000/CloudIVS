use std::net::IpAddr;
use std::collections::HashMap;


/**
 * Transport protocol.
 */
#[derive(PartialEq, Display, Debug, EnumString)]
enum TransportProtocol {
    #[strum(serialize = "udp")]
    UDP,

    #[strum(serialize = "tcp")]
    TCP,
}

/**
 * Valid types for 'trace' event.
 */
#[derive(PartialEq, Display, Debug, EnumString)]
enum TransportTraceEventType {
    #[strum(serialize = "probation")]
    PROBATION,

    #[strum(serialize = "bwe")]
    BWE,
}


/**
 * Valid types for 'trace' event.
 */
#[derive(PartialEq, Display, Debug, EnumString)]
enum TransportTraceEventDirection {
    #[strum(serialize = "in")]
    IN,

    #[strum(serialize = "out")]
    OUT,
}

/**
 * Valid types for 'trace' event.
 */
#[derive(PartialEq, Display, Debug, EnumString)]
enum SctpState {
    #[strum(serialize = "new")]
    NEW,

    #[strum(serialize = "connecting")]
    CONNECTING,

    #[strum(serialize = "connected")]
    CONNECTED,

    #[strum(serialize = "failed")]
    FAILED,

    #[strum(serialize = "closed")]
    CLOSED,

}

struct TransportListenIp {
    /**
	 * Listening IPv4 or IPv6.
	 */
    ip: IpAddr,

    /**
	 * Announced IPv4 or IPv6 (useful when running mediasoup behind NAT with
	 * private IP).
	 */
    announcedIp: Option<IpAddr>,
}

struct TransportTuple {
    localIp: IpAddr,
    localPort: u32,
    remoteIp: Option<IpAddr>,
    remotePort: Option<u32>,
    protocol: TransportProtocol,
}


struct TransportTraceEventData {
    /**
     * Trace type.
     */
    eventType: TransportTraceEventType,

    /**
     * Event timestamp.
     */
    timestamp: u64,

    /**
     * Event direction.
     */
    direction: TransportTraceEventDirection,

    /**
     * Per type information.
     */
    info: HashMap<String, String>,
}


// struct Transport {
//
// }
//
// impl parallel_event_emitter