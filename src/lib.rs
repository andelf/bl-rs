use crc::{Crc, CRC_32_ISO_HDLC};

pub mod commands;
pub mod error;
pub mod transport;

pub mod fw_header;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
