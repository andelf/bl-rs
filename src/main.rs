use std::{env, os, time::Duration};

use anyhow::Result;
use crc::{Crc, CRC_32_ISO_HDLC};
use serialport::SerialPort;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub trait Transport {
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>>;

    fn write_bytes(&mut self, buf: &[u8]) -> Result<()>;

    /// len for UART
    fn read_u16(&mut self) -> Result<u16> {
        let buf = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(buf[..].try_into().unwrap()))
    }

    /// len for SDIO
    fn read_u32(&mut self) -> Result<u32> {
        let buf = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(buf[..].try_into().unwrap()))
    }

    fn call_raw_no_resp(&mut self, cmd: &[u8]) -> Result<()> {
        self.write_bytes(cmd)?;
        let ack = self.read_bytes(2)?;
        if ack == b"FL" {
            let code = self.read_u16()?;
            anyhow::bail!("call_raw_cmd: FL code = {:04x}", code);
        }
        if ack != b"OK" {
            anyhow::bail!("call_raw_cmd: ack != OK");
        }
        Ok(())
    }

    fn call_raw_cmd(&mut self, cmd: &[u8]) -> Result<Vec<u8>> {
        self.write_bytes(cmd)?;
        let ack = self.read_bytes(2)?;
        if ack == b"FL" {
            let code = self.read_u16()?;
            anyhow::bail!("call_raw_cmd: FL code = {:04x}", code);
        }
        if ack != b"OK" {
            anyhow::bail!("call_raw_cmd: ack != OK");
        }
        let len_payload = self.read_u16()?;
        println!("payload => {}", len_payload);
        let payload = self.read_bytes(len_payload as usize)?;
        Ok(payload)
    }
}

impl Transport for Box<dyn SerialPort> {
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; n];
        let nread = self.read(&mut buf[..])?;
        if nread != n {
            return Err(anyhow::anyhow!("read_bytes: nread != n"));
        }
        println!("I: read {} => {:02x?}", n, buf);
        Ok(buf)
    }

    fn write_bytes(&mut self, buf: &[u8]) -> Result<()> {
        let n = self.write(buf)?;
        if n != buf.len() {
            return Err(anyhow::anyhow!("write_bytes: n != buf.len()"));
        }
        println!("I: write {} => {:02x?}", n, buf);
        Ok(())
    }
}

fn main() -> Result<()> {
    let dev = env::args()
        .nth(1)
        .expect("usage: prog /dev/tty.usbserial-0001");
    let mut serial = serialport::new(&dev, 115200)
        .open()
        .expect("failed to open port");
    serial.set_timeout(Duration::from_secs(1)).unwrap();

    let mut buf = [0u8; 256];

    // send uart sync bytes..
    let sync_len = (0.006 * 115200.0 / 10.0) as usize;
    println!("sync_len: {}", sync_len);
    let sync_bytes = vec![0x55_u8; sync_len];

    serial.write(&sync_bytes).unwrap();
    let n = serial.read(&mut buf[..]).unwrap();

    println!("sync <= {:?}", &buf[..n]);
    let payload = &buf[..n];
    if payload.ends_with(b"OK") {
        println!("baudrate sync OK");
    }

    // get_boot_info
    serial.write(b"\x10\x00\x00\x00").unwrap();

    let ack = serial.read_bytes(2).unwrap();

    if ack != b"OK" {
        anyhow::bail!("get_boot_info: ack != OK");
    }
    let len_payload = serial.read_u16()?;
    println!("payload => {}", len_payload);
    let boot_info = serial.read_bytes(len_payload as usize)?;
    println!("get_boot_info => {:02x?}", boot_info);

    println!(
        "BootRom version {}.{}.{}.{}",
        boot_info[0], boot_info[1], boot_info[2], boot_info[3]
    );

    let sign = boot_info[4];
    let encrypt = boot_info[5];
    println!("sign => {}", sign);
    println!("encrypt => {}", encrypt);

    // BL616, 6bytes, other 8bytes
    let chip_id = [
        boot_info[17],
        boot_info[16],
        boot_info[15],
        boot_info[14],
        boot_info[13],
        boot_info[12],
    ];
    println!("chip_id => {:02x?}", chip_id);

    // get_chip_id
    serial.write(b"\x05\x00\x00\x00").unwrap();
    let ack = serial.read_bytes(2).unwrap();

    if ack != b"OK" {
        anyhow::bail!("get_boot_info: ack != OK");
    }

    let len_payload = serial.read_u16()?;
    println!("payload => {}", len_payload);
    let chip_id = serial.read_bytes(len_payload as usize)?;
    let chip_id = String::from_utf8(chip_id);
    // 43484950574230334130305f424c0000
    // "CHIPWB03A00_BL\x00\x00"
    println!("get_chip_id => {:?}", chip_id);

    // Clock PLL set. clk_set
    let _ = serial.call_raw_no_resp(b"\x22\xcc\x08\x00\x01\x00\x00\x00\x00\xc2\x01\x00")?;

    // efuse_read_mac
    let efuse = serial.call_raw_cmd(b"\x42\x00\x00\x00")?;
    let checksum = CRC32.checksum(&efuse[..6]);
    println!("crc32 {:08x}", checksum);
    assert_eq!(
        checksum.to_le_bytes(),
        efuse[6..10],
        "MAC checksum mismatch"
    );
    let mac_addr = &efuse[..6];
    println!("MAC: {:02x?}", mac_addr);

    // flash_read_jid
    let jedec_id = serial.call_raw_cmd(b"\x36\x00\x00\x00")?;
    println!("Jedec ID: {:02x?}", jedec_id);

    println!("Hello, world!");
    Ok(())
}
