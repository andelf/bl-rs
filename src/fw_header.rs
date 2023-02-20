pub mod bl616;

/// 256 byte efuse
pub struct FwHeader(bl616::bootheader_t);
