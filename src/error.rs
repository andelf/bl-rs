use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Custom(String),
    #[error("Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),
    #[error("BFLB error: {0:04x}")]
    Code(u16),
    #[error("Eflash Loader error: {0:04x}")]
    FlashLoader(u16),
    #[error("UTF8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("CRC checksum error")]
    Checksum,
}
