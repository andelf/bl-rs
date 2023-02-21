pub mod bl616;

/// 256 byte efuse
// BFNP
/*

 *(.text.entry)
    KEEP (*(SORT_NONE(.init)))
    KEEP (*(SORT_NONE(.vector)))


__RFTLV_SIZE_OFFSET = 1K;
__RFTLV_SIZE_HOLE = 2K;
__RFTLV_HEAD1_H = (0x46524C42); /* BLRF */
__RFTLV_HEAD1_L = (0x41524150); /* PAPA */

. = ORIGIN(xip_memory) + __RFTLV_SIZE_OFFSET + __RFTLV_SIZE_HOLE;

. __text_code_start__ = .;

    *(.text)
    *(.text.*)



 */

pub struct FwHeader(bl616::bootheader_t);
