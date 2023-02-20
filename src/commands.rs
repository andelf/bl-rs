use std::{fmt, ops};

use crate::{
    error::{Error, Result},
    CRC32,
};

pub trait Response: Sized {
    fn from_raw(raw: &[u8]) -> Result<Self>;

    fn size_hint() -> Option<usize> {
        None
    }
}

pub trait Command {
    type Response: Response;
    fn command_id(&self) -> u8;
    fn to_raw(&self) -> Vec<u8> {
        vec![self.command_id(), 0x00, 0x00, 0x00]
    }
}

impl Response for () {
    fn from_raw(_raw: &[u8]) -> Result<Self> {
        Ok(())
    }

    fn size_hint() -> Option<usize> {
        Some(0)
    }
}

impl Response for Vec<u8> {
    fn from_raw(raw: &[u8]) -> Result<Self> {
        Ok(raw.to_vec())
    }
}

impl Response for String {
    fn from_raw(raw: &[u8]) -> Result<Self> {
        Ok(String::from_utf8(raw.to_vec())?)
    }
}

#[derive(Debug)]
pub struct Crc32<T> {
    pub data: T,
}
impl<T: Response> Response for Crc32<T> {
    fn from_raw(raw: &[u8]) -> Result<Self> {
        let len = raw.len();
        if len < 4 {
            return Err(Error::Custom(format!("len < 4 {:?}", raw)));
        }
        let crc = CRC32.checksum(&raw[..len - 4]);
        let crc_raw = u32::from_le_bytes(raw[len - 4..len].try_into().unwrap());
        if crc != crc_raw {
            return Err(Error::Checksum);
        }
        let data = T::from_raw(&raw[4..])?;
        Ok(Self { data })
    }
}

impl<T> ops::Deref for Crc32<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug)]
pub struct GetBootInfo;

pub struct BootInfo {
    pub boot_rom_version: [u8; 4],
    pub sign: u8,
    pub encrypt: u8,
    pub chip_id: Vec<u8>,
}
impl Command for GetBootInfo {
    type Response = BootInfo;
    fn command_id(&self) -> u8 {
        0x10
    }
}
impl Response for BootInfo {
    fn from_raw(raw: &[u8]) -> Result<Self> {
        let mut chip_id = raw[12..18].to_vec();
        chip_id.reverse();
        Ok(Self {
            boot_rom_version: raw[0..4].try_into().unwrap(),
            sign: raw[4],
            encrypt: raw[5],
            chip_id: chip_id,
        })
    }
}
impl fmt::Debug for BootInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootInfo")
            .field(
                "boot_rom_version",
                &self
                    .boot_rom_version
                    .iter()
                    .map(|b| format!("{}", b))
                    .collect::<Vec<String>>()
                    .join("."),
            )
            .field("sign", &self.sign)
            .field("encrypt", &self.encrypt)
            // b40ecf35affb
            .field(
                "chip_id",
                &self
                    .chip_id
                    .iter()
                    .map(|b| format!("{:02x?}", b))
                    .collect::<Vec<String>>()
                    .join(""),
            )
            .finish()
    }
}

#[derive(Debug)]
pub struct GetChipId;
impl Command for GetChipId {
    type Response = String;
    fn command_id(&self) -> u8 {
        0x05
    }
}

#[derive(Debug)]
pub struct ClockSet {
    irq_enable: bool,
    //115200 => 0x0001c200 => \x00\xc2\x01\x00
    load_speed: u32,
    clock_parameter: Vec<u8>,
}
impl Default for ClockSet {
    fn default() -> Self {
        Self {
            irq_enable: true,
            load_speed: 115200,
            clock_parameter: vec![],
        }
    }
}
// \x22\xcc\x08\x00\x01\x00\x00\x00\x00\xc2\x01\x00
impl Command for ClockSet {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x22
    }
    fn to_raw(&self) -> Vec<u8> {
        let mut raw = vec![self.command_id(), 0x00, 0x00, 0x00];

        raw.extend_from_slice(&(self.irq_enable as u32).to_le_bytes());
        raw.extend_from_slice(&self.load_speed.to_le_bytes());
        raw.extend_from_slice(&self.clock_parameter);

        // calc checksum
        let len = (raw[4..].len() as u16).to_le_bytes();
        raw[2] = len[0];
        raw[3] = len[1];

        let checksum = raw[2..]
            .iter()
            .fold(0_u8, |acc, &c| acc.overflowing_add(c).0);

        raw[1] = checksum;
        raw
    }
}

pub struct Reset;
impl Command for Reset {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x21
    }
}

pub struct EfuseReadMac;
impl Command for EfuseReadMac {
    type Response = Crc32<Vec<u8>>;
    fn command_id(&self) -> u8 {
        0x42
    }
}

pub struct FlashReadJedecId;
impl Command for FlashReadJedecId {
    type Response = Vec<u8>;
    fn command_id(&self) -> u8 {
        0x36
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_set() {
        let clock_set = ClockSet {
            irq_enable: true,
            load_speed: 115200,
            clock_parameter: vec![],
        };
        let raw = clock_set.to_raw();
        assert_eq!(
            raw,
            vec![0x22, 0xcc, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0xc2, 0x01, 0x00]
        );
    }
}
