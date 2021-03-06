use std::io::{Read, Write};

use crate::proto::common::sync_io::{ReadExt, WriteExt};
use crate::proto::error::ErrorKind;
use crate::proto::error::Error;
use crate::proto::rtp::constants::RTP_VERSION;
use crate::proto::traits::Result;
use crate::proto::types::U5;

pub fn write_common<W: Write>(
    writer: &mut W,
    packet_type: u8,
    fb_message_type: U5,
    payload: &[u8],
) -> Result<()> {
    track_assert_eq!(payload.len() % 4, 0, ErrorKind::Invalid);

    track!(writer.write_u8(RTP_VERSION << 6 | fb_message_type).map_err(Error::from));
    track!(writer.write_u8(packet_type).map_err(Error::from));

    let word_count = payload.len() / 4;
    track_assert!(word_count < 0x10000, ErrorKind::Invalid);

    track!(writer.write_u16be(word_count as u16).map_err(Error::from));
    track!(writer.write_all(payload).map_err(Error::from));

    Ok(())
}

pub fn read_common<R: Read>(reader: &mut R, expected_type: u8) -> Result<(U5, Vec<u8>)> {
    let b = track_try_unwrap!(reader.read_u8().map_err(Error::from));
    track_assert_eq!(
        b >> 6,
        RTP_VERSION,
        ErrorKind::Unsupported,
        "Unsupported RTP version: {}",
        b >> 6
    );
    let padding = (b & 0b0010_0000) != 0;
    let fb_message_type = b & 0b0001_1111;

    let packet_type = track_try_unwrap!(reader.read_u8().map_err(Error::from));
    track_assert_eq!(
        packet_type,
        expected_type,
        ErrorKind::Invalid,
        "Unexpected SCTP packet type: actual={}, expected={}",
        packet_type,
        expected_type
    );

    let word_count = track_try_unwrap!(reader.read_u16be().map_err(Error::from)) as usize;
    let mut payload = track_try_unwrap!(reader.read_bytes(word_count * 4).map_err(Error::from));
    if padding {
        let payload_len = payload.len();
        track_assert_ne!(payload_len, 0, ErrorKind::Invalid);

        let padding_len = payload[payload_len - 1] as usize;
        track_assert!(padding_len <= payload.len(), ErrorKind::Invalid);

        payload.truncate(payload_len - padding_len);
    }
    track_assert_eq!(payload.len() % 4, 0, ErrorKind::Invalid);

    Ok((fb_message_type, payload))
}