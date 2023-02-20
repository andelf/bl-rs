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

    println!("Hello, world!");

    Ok(())
}
