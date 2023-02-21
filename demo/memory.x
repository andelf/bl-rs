__RFTLV_SIZE_OFFSET = 1K;
__RFTLV_SIZE_HOLE = 2K;
__RFTLV_HEAD1_H = (0x46524C42); /* BLRF */
__RFTLV_HEAD1_L = (0x41524150); /* PAPA */

__J_0XC00 = (0x4010006f); /* j 0xc00 */


MEMORY
{
    /* FLASH_HEADER (rx) : ORIGIN = 0xA0000000, LENGTH = 1K */

    FLASH     (rxa!w) : ORIGIN = 0xA0000000, LENGTH = 4M
    HBNRAM    (wxa)   : ORIGIN = 0x20010000, LENGTH = 4K
    ITCM_OCRAM (wxa)  : ORIGIN = 0x62FC0000, LENGTH = 20K
    DTCM_OCRAM (wrx)  : ORIGIN = 0x62FC5000, LENGTH = 4K
    OCRAM (!rx)       : ORIGIN = 0x62FC6000, LENGTH = 320K - 20K - 4K
}

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", OCRAM);
REGION_ALIAS("REGION_BSS", OCRAM);
REGION_ALIAS("REGION_HEAP", OCRAM);
REGION_ALIAS("REGION_STACK", DTCM_OCRAM);

/* The device stores RF configuration in 0x400-0xc00 so we place .text after that */
_stext = ORIGIN(FLASH) + 0xC00;


SECTIONS {
    .text.entry :
    {
        . = ORIGIN(FLASH);
        LONG(__J_0XC00);
    } > FLASH

    .rftlv.tool :
    {
        . = ORIGIN(FLASH) + __RFTLV_SIZE_OFFSET;
        PROVIDE( _ld_symbol_rftlv_address = . );
        LONG(__RFTLV_HEAD1_H);
        LONG(__RFTLV_HEAD1_L);
        . = ORIGIN(FLASH) + __RFTLV_SIZE_OFFSET + __RFTLV_SIZE_HOLE;
    } > FLASH
}


