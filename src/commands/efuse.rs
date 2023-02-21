use super::{Command, Crc32};

pub struct EfuseReadMac;
impl Command for EfuseReadMac {
    type Response = Crc32<Vec<u8>>;
    fn command_id(&self) -> u8 {
        0x42
    }
}
