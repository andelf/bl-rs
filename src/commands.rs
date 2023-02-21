use std::{fmt, ops};

use crate::{
    error::{Error, Result},
    CRC32,
};

pub use self::efuse::*;

mod efuse;

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

        recalc_checksum(&mut raw);

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

// Erase flash from 0x2000 to 0x8d3f
// if_write 30f40800 00200000 3f8d0000
pub struct FlashErase {
    pub start: u32,
    pub end: u32,
}
impl Command for FlashErase {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x30
    }
    fn to_raw(&self) -> Vec<u8> {
        let mut raw = vec![self.command_id(), 0x00, 0x00, 0x00];

        raw.extend_from_slice(&self.start.to_le_bytes());
        raw.extend_from_slice(&self.end.to_le_bytes());

        recalc_checksum(&mut raw);

        raw
    }
}

pub struct FlashWrite {
    pub start_addr: u32,
    // 2K
    pub data: Vec<u8>,
}
impl Command for FlashWrite {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x31
    }
    fn to_raw(&self) -> Vec<u8> {
        let mut raw = vec![self.command_id(), 0x00, 0x00, 0x00];

        raw.extend_from_slice(&self.start_addr.to_le_bytes());
        raw.extend_from_slice(&self.data);

        recalc_checksum(&mut raw);

        raw
    }
}

pub struct FlashRead {
    pub start_addr: u32,
    pub len: u32,
}
impl Command for FlashRead {
    type Response = Vec<u8>;
    fn command_id(&self) -> u8 {
        0x32
    }
    fn to_raw(&self) -> Vec<u8> {
        let mut raw = vec![self.command_id(), 0x00, 0x00, 0x00];

        raw.extend_from_slice(&self.start_addr.to_le_bytes());
        raw.extend_from_slice(&self.len.to_le_bytes());

        recalc_checksum(&mut raw);

        raw
    }
}

pub struct FlashReadJedecId;
impl Command for FlashReadJedecId {
    type Response = Vec<u8>;
    fn command_id(&self) -> u8 {
        0x36
    }
}

pub struct FlashReadStatusReg {
    cmd: u32,
    len: u32,
}
// TODO

pub struct FlashWriteCheck;
impl Command for FlashWriteCheck {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x3a
    }
}

/// set flash parameter
pub struct FlashSetPara {
    //bit 7 flash pin set from efuse flash cfg
    //bit 6 flash select 0: flash1, 1: flash2
    //bit 5-0 flash pin cfg:
    //0x0: single flash, sf1 internal swap io3 and io0
    //0x1: single flash, sf1 internal swap io3 with io0 and io2 with cs
    //0x2: single flash, sf1 internal no swap
    //0x3: single flash, sf1 internal swap io2 with cs
    //0x4: single flash, sf2 external GPIO4-9 and swap io3 with io0
    //0x8: single flash, sf3 external GPIO10-15
    //0x14:dual flash, sf1 internal swap io3 and io0, sf2 external GPIO4-9 swap io3 with io0
    //0x15:dual flash, sf1 internal swap io3 with io0 and io2 with cs, sf2 external GPIO4-9 swap io3 with io0
    //0x16:dual flash, sf1 internal no swap, sf2 external GPIO4-9 swap io3 with io0
    //0x17:dual flash, sf1 internal swap io2 with cs, sf2 external GPIO4-9 swap io3 with io0
    //0x24:single flash, sf2 external GPIO4-9
    //0x34:dual flash, sf1 internal swap io3 and io0, sf2 external GPIO4-9 no swap
    //0x35:dual flash, sf1 internal swap io3 with io0 and io2 with cs, sf2 external GPIO4-9 no swap
    //0x36:dual flash, sf1 internal no swap, sf2 external GPIO4-9 no swap
    //0x37:dual flash, sf1 internal swap io2 with cs, sf2 external GPIO4-9 no swap
    //0x80:flash pin set from efuse flash cfg
    /* 0x0101ff is default set: flash_io_mode=1, flash_clock_cfg=1, flash_pin=0xff */
    flash_pin: u8,
    // bit 7-4 flash_clock_type: 0:120M wifipll, 1:xtal, 2:128M cpupll, 3:80M wifipll, 4:bclk, 5:96M wifipll
    // bit 3-0 flash_clock_div
    flash_clock_cfg: u8,
    // 0:NIO, 1:DO, 2:QO, 3:DIO, 4:QIO
    flash_io_mode: u8,
    // #0:0.5T delay, 1:1T delay, 2:1.5T delay, 3:2T delay
    flash_clk_delay: u8,
    flash_para: Vec<u8>,
}
impl Default for FlashSetPara {
    fn default() -> Self {
        Self {
            flash_pin: 0x02,
            flash_clock_cfg: 0x41,
            flash_io_mode: 0x01,
            flash_clk_delay: 0x00,
            flash_para: include_bytes!("../chips/bl616/flash_para.bin").to_vec(),
        }
    }
}

impl Command for FlashSetPara {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x3b
    }
    fn to_raw(&self) -> Vec<u8> {
        let mut raw = vec![self.command_id(), 0x00, 0x00, 0x00];

        /* flash_set = (flash_pin << 0) +\
        (flash_clock_cfg << 8) +\
        (flash_io_mode << 16) +\
        (flash_clk_delay << 24) */
        raw.extend_from_slice(&[
            self.flash_pin,
            self.flash_clock_cfg,
            self.flash_io_mode,
            self.flash_clk_delay,
        ]);
        raw.extend_from_slice(&self.flash_para);
        recalc_checksum(&mut raw);
        raw
    }
}

pub struct FlashXipReadSha {
    pub start_addr: u32,
    pub len: u32,
}
impl Command for FlashXipReadSha {
    type Response = Vec<u8>; // 32 byte SHA checksum
    fn command_id(&self) -> u8 {
        0x3e
    }
    fn to_raw(&self) -> Vec<u8> {
        let mut raw = vec![self.command_id(), 0x00, 0x00, 0x00];

        raw.extend_from_slice(&self.start_addr.to_le_bytes());
        raw.extend_from_slice(&self.len.to_le_bytes());

        recalc_checksum(&mut raw);

        raw
    }
}

// xip mode Verify
pub struct FlashXipReadStart;
impl Command for FlashXipReadStart {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x60
    }
}

pub struct FlashXipReadFinish;
impl Command for FlashXipReadFinish {
    type Response = ();
    fn command_id(&self) -> u8 {
        0x61
    }
}


pub struct LogRead;
impl Command for LogRead {
    type Response = String;
    fn command_id(&self) -> u8 {
        0x71
    }
}


/// Fill header length and checksum
fn recalc_checksum(raw: &mut [u8]) {
    if raw.len() <= 4 {
        return;
    }
    let len = (raw[4..].len() as u16).to_le_bytes();
    raw[2] = len[0];
    raw[3] = len[1];

    let checksum = raw[2..]
        .iter()
        .fold(0_u8, |acc, &c| acc.overflowing_add(c).0);

    raw[1] = checksum;
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

    #[test]
    fn erase_flash() {
        // Erase flash from 0x2000 to 0x8d3f
        // if_write 30f40800 00200000 3f8d0000
        let erase_flash = FlashErase {
            start: 0x2000,
            end: 0x8d3f,
        };
        let raw = erase_flash.to_raw();
        assert_eq!(
            raw,
            vec![0x30, 0xf4, 08, 0x00, 0x00, 0x20, 0x00, 0x00, 0x3f, 0x8d, 0x00, 0x00]
        );
    }
}
