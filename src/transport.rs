use serialport::SerialPort;

use crate::commands::{Command, Response};
use crate::error::{Error, Result};

pub trait Transport {
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>>;

    fn write_bytes(&mut self, buf: &[u8]) -> Result<()>;

    fn send_command<C: Command>(&mut self, cmd: C) -> Result<C::Response> {
        let raw = cmd.to_raw();
        self.write_bytes(&raw)?;
        let ack = self.read_bytes(2)?;
        if ack == b"FL" {
            let code = self.read_u16()?;
            return Err(Error::Code(code));
        }
        if ack != b"OK" {
            return Err(Error::Custom(format!("ack != OK {:?}", ack)));
        }
        if C::Response::size_hint() == Some(0) {
            return Ok(C::Response::from_raw(&[])?);
        }
        let len_payload = self.read_u16()?;
        if len_payload != 0 {
            let payload = self.read_bytes(len_payload as usize)?;
            C::Response::from_raw(&payload)
        } else {
            Ok(C::Response::from_raw(&[])?)
        }
    }

    /// len for UART
    fn read_u16(&mut self) -> Result<u16> {
        let buf = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(buf[..].try_into().expect("won't fail")))
    }

    /// len for SDIO
    fn read_u32(&mut self) -> Result<u32> {
        let buf = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(buf[..].try_into().expect("won't fail")))
    }

    fn call_raw_no_resp(&mut self, cmd: &[u8]) -> Result<()> {
        self.write_bytes(cmd)?;
        let ack = self.read_bytes(2)?;
        if ack == b"FL" {
            let code = self.read_u16()?;
            return Err(Error::Code(code));
        }
        if ack != b"OK" {
            return Err(Error::Custom(format!("ack != OK {:?}", ack)));
        }
        Ok(())
    }

    fn call_raw_cmd(&mut self, cmd: &[u8]) -> Result<Vec<u8>> {
        self.write_bytes(cmd)?;
        let ack = self.read_bytes(2)?;
        if ack == b"FL" {
            let code = self.read_u16()?;
            return Err(Error::Code(code));
        }
        if ack != b"OK" {
            return Err(Error::Custom(format!("ack != OK {:?}", ack)));
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
            return Err(Error::Custom(format!("read_bytes: nread != n")));
        }
        println!("D: read {} => {:02x?}", n, buf);
        Ok(buf)
    }

    fn write_bytes(&mut self, buf: &[u8]) -> Result<()> {
        let n = self.write(buf)?;
        if n != buf.len() {
            return Err(Error::Custom("write_bytes: n != buf.len()".to_string()));
        }
        println!("D: write {} => {}", n, hex::encode(&buf));
        Ok(())
    }
}
