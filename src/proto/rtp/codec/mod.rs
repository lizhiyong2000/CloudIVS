use crate::proto::rtsp::codec::{CodecEvent, ProtocolError};
use futures::channel::mpsc::UnboundedSender;
use tokio_util::codec::{Decoder, Encoder};
use bytes::BytesMut;
use crate::proto::rtp::rtp::{RtpPacket, RtpPacketReader};
use crate::proto::rtp::mutex::{MuxedPacket, MuxPacketReader};
use crate::proto::rtcp::rtcp_packet::{RtcpCompoundPacket, RtcpPacket, RtcpPacketReader};
use crate::proto::traits::{WriteTo, ReadPacket};
use crate::proto::error::{Error, ErrorKind};

#[derive(Debug)]
pub struct Codec {

    /// An event sink that is sent [`CodecEvent`]s. For example, whenever decoding starts, an event
    /// will be sent for that.
    tx_event: Option<UnboundedSender<CodecEvent>>,

    packet_reader:MuxPacketReader<RtpPacketReader, RtcpPacketReader>,
}

impl Codec {


    /// Constructs a new codec without an event sink.
    pub fn new() -> Self {
        Codec {
            packet_reader: MuxPacketReader::new(RtpPacketReader, RtcpPacketReader),
            tx_event: None,
        }
    }

    /// Sends a [`CodecEvent`] through the internal event sink.
    ///
    /// If an error is encountered while sending the codec event, then no more events will be sent
    /// for the duration of this codec's lifetime.
    fn send_codec_event(&mut self, event: CodecEvent) {
        if let Some(tx_event) = self.tx_event.as_ref() {
            if tx_event.unbounded_send(event).is_err() {
                self.tx_event = None;
            }
        }
    }

    /// Constructs a new codec with an event sink.
    pub fn with_events(tx_event: UnboundedSender<CodecEvent>) -> Self {
        Codec {
            packet_reader: MuxPacketReader::new(RtpPacketReader, RtcpPacketReader),
            tx_event: Some(tx_event),
        }
    }
}

impl Decoder for Codec {
    type Item = MuxedPacket<RtpPacket, RtcpCompoundPacket<RtcpPacket>>;
    type Error = Error;

    /// Decodes a message.
    ///
    /// Using the internal decoders, this function will attempt to make progress on decoding either
    /// a request or response using the buffer. If neither of the decoders are active, this
    /// function will send a [`CodecEvent::DecodingStarted`] event if the buffer is non-empty after
    /// removing all preceding newlines.
    ///
    /// The return value of this function can be divided into four parts:
    ///
    /// * If there was enough data provided to successfully decode a message, then
    ///   `Ok(Some(`[`Message`]`))` will be returned.
    /// * If there was not enough data but no error occurred, then `Ok(None)` will be returned
    ///   indicating that more data is needed.
    /// * If the decoder encountered an error, then `Err(`[`ProtocolError`]`)` will be returned.
    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need to determine whether we are trying to decode a request or response. If either of the
        // internal decoder states are past their starting states, then we continue off of that.
        // Otherwise, we check if the message starts with `"RTSP/"` which indicates that it is a
        // response. If not, it is a request.
        self.send_codec_event(CodecEvent::EncodingStarted);
        let result = self.packet_reader.read_packet(buffer);
        self.send_codec_event(CodecEvent::EncodingEnded);
        match result{
            Ok(t)=> Ok(Some(t)),
            Err(e) => Err(e),
        }
    }

    /// Called when there are no more bytes available to be read from the underlying I/O.
    ///
    /// This function will attempt to decode a message as described in [`Codec::decode`]. If there
    /// is not enough data to do so, then `Err(`[`ProtocolError::UnexpectedEOF`]`)` will be
    /// returned.
    fn decode_eof(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.decode(buffer)? {
            Some(message) => Ok(Some(message)),
            None => {
                if buffer.is_empty() {
                    Ok(None)
                } else {
                    track_panic!( ErrorKind::Other,  "UnexpectedEOF")
                    // Err(Error(ErrorKind::Other))
                }
            }
        }
    }
}

impl Default for Codec {
    fn default() -> Self {
        Codec::new()
    }
}

impl Encoder<MuxedPacket<RtpPacket, RtcpCompoundPacket<RtcpPacket>>> for Codec {
    // type Item = Message;
    type Error = Error;

    /// Encodes a message.
    ///
    /// This function will encode the given message into the given buffer. Before encoding the
    /// message, a [`CodecEvent::EncodingStarted`] event will be sent. And after encoding has
    /// finished, an [`CodecEvent::EncodingEnded`] event will be sent.
    ///
    /// Although a [`Result`] is returned, this function will never return an error as the actual
    /// message encoding cannot fail. As a result, `Ok(())` will always be returned.
    fn encode(&mut self, message: MuxedPacket<RtpPacket, RtcpCompoundPacket<RtcpPacket>>, buffer: &mut BytesMut) -> Result<(), Self::Error> {
        self.send_codec_event(CodecEvent::EncodingStarted);

        match message {
            MuxedPacket::Rtp(rtp) => rtp.write_to(buffer),
            MuxedPacket::Rtcp(rtcp) => rtcp.write_to(buffer),
            _ => {}
        }

        self.send_codec_event(CodecEvent::EncodingEnded);
        Ok(())
    }
}
