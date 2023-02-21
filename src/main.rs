use std::{env, os, time::Duration};

use anyhow::Result;
use bl::{
    commands::{self, Command},
    transport::Transport,
};
use crc::{Crc, CRC_32_ISO_HDLC};
use serialport::SerialPort;

fn main() -> Result<()> {
    let dev = env::args()
        .nth(1)
        .expect("usage: prog /dev/tty.usbserial-0001 firmware.bin");
    let fname = env::args()
        .nth(2)
        .expect("usage: prog /dev/tty.usbserial-0001 firmware.bin");

    let mut firmware = std::fs::read(fname)?;
    if firmware.len() % 16 != 0 {
        firmware.resize(firmware.len() + 16 - firmware.len() % 16, 0);
    }

    println!("Firmware size: {}", firmware.len());

    let mut serial = serialport::new(&dev, 115200)
        .open()
        .expect("failed to open port");
    serial.set_timeout(Duration::from_secs(10)).unwrap();

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

    let boot_info = serial.send_command(commands::GetBootInfo)?;
    println!("boot info => {:?}", boot_info);
    // get_boot_info

    // 43484950574230334130305f424c0000
    //"CHIPWB03A00_BL\x00\x00"
    let chip_id = serial.send_command(commands::GetChipId)?;
    println!("chip id {:?}", chip_id);

    // Clock PLL set. clk_set
    let _ = serial.send_command(commands::ClockSet::default())?;

    let mac_addr = serial.send_command(commands::EfuseReadMac)?;
    println!("mac_addr => {:02x?}", mac_addr);

    let jedec_id = serial.send_command(commands::FlashReadJedecId)?;
    println!("jedec_id => {:02x?}", jedec_id);

    serial.send_command(commands::FlashSetPara::default())?;

    // flash load
    // Erase flash from 0x2000 to 0x8d3f
    serial.send_command(commands::FlashErase {
        start: 0x0000_2000,
        end: 0x0000_8d3f,
    })?;


    let mut start_addr = 0x2000;
    for chunk in firmware.chunks(2*1024) {
        let len = chunk.len();
        let end_addr = start_addr + len as u32 - 1;
        println!("flash write {:04x}..{:04x}", start_addr, end_addr);
        serial.send_command(commands::FlashWrite {
            start_addr,
            data: chunk.to_vec(),
        })?;
        start_addr += len as u32;
    }
    println!("Flash done {}", firmware.len());

    // write check
    serial.send_command(commands::FlashWriteCheck)?;
    serial.send_command(commands::FlashXipReadStart)?;
    let sha_address = serial.send_command(commands::FlashXipReadSha {
        start_addr: 0x0000_2000,
        len: 0x0000_6d40,
    })?;
    println!("=> {:02x?}", sha_address);

    serial.send_command(commands::FlashXipReadFinish)?;

    println!("Hello, world!");
/*
    let ret = serial.send_command(commands::FlashRead {
        start_addr: 0x0000_0000,
        len: 0x0000_1000,
    })?;
*/

    // let ret = serial.send_command(commands::LogRead)?;
    // println!("log read => {:02x?}", ret);

    serial.send_command(commands::Reset)?;
    // bootinfo.bin
    // chips/bl616/img_create_mcu/bootinfo.bin

    // img.bin
    // chips/bl616/img_create_mcu/img.bin

    Ok(())
}
