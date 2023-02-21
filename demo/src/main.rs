#![no_std]
#![no_main]
#![allow(dead_code, non_camel_case_types, non_snake_case)]

use core::mem;
use core::ptr;

use bl616_pac as pac;
use pac::glb::gpio_config::PIN_MODE_A;
use panic_halt as _;
use riscv::register::mhartid;

#[riscv_rt::pre_init]
unsafe fn pre_init() {}

static mut ROM_DRIVER_TABLE: Option<&[u32; 512]> = None;

// TODO: ATTR_TCM_SECTION
unsafe fn bflb_flash_init() {}

#[repr(C)]
pub enum XTALType {
    None,           /* XTAL is none */
    GLB_XTAL_24M,   /* XTAL is 24M */
    GLB_XTAL_32M,   /* XTAL is 32M */
    GLB_XTAL_38P4M, /* XTAL is 38.4M */
    GLB_XTAL_40M,   /* XTAL is 40M */
    GLB_XTAL_26M,   /* XTAL is 26M */
    GLB_XTAL_RC32M, /* XTAL is RC32M */
}

#[repr(C)]
pub enum PLLType {
    None = 0,            /* power on xtal and pll */
    GLB_PLL_WIFIPLL = 1, /* power on WIFIPLL */
    GLB_PLL_AUPLL = 2,   /* power on AUPLL */
}

#[repr(C)]
pub enum GLB_MCU_SYS_CLK_Type {
    GLB_MCU_SYS_CLK_RC32M,            /* use RC32M as system clock frequency */
    GLB_MCU_SYS_CLK_XTAL,             /* use XTAL as system clock */
    GLB_MCU_SYS_CLK_TOP_AUPLL_DIV2,   /* use TOP_AUPLL_DIV2 output as system clock */
    GLB_MCU_SYS_CLK_TOP_AUPLL_DIV1,   /* use TOP_AUPLL_DIV1 output as system clock */
    GLB_MCU_SYS_CLK_TOP_WIFIPLL_240M, /* use TOP_WIFIPLL_240M output as system clock */
    GLB_MCU_SYS_CLK_TOP_WIFIPLL_320M, /* use TOP_WIFIPLL_320M output as system clock */
}

#[repr(C)]
pub enum BL_MTimer_Source_Clock_Type {
    BL_MTIMER_SOURCE_CLOCK_MCU_XCLK, /* MCU xclk clock */
    BL_MTIMER_SOURCE_CLOCK_MCU_CLK,  /* MCU root clock */
}

unsafe fn system_clock_init() {
    let GLB_Power_On_XTAL_And_PLL_CLK: unsafe extern "C" fn(
        xtalType: XTALType,
        pllType: i32,
    ) -> *const i32 = mem::transmute(ROM_DRIVER_TABLE.unwrap()[115]);
    let GLB_Set_MCU_System_CLK: unsafe extern "C" fn(clkFreq: GLB_MCU_SYS_CLK_Type) -> *const i32 =
        mem::transmute(ROM_DRIVER_TABLE.unwrap()[144]);
    let CPU_Set_MTimer_CLK: unsafe extern "C" fn(
        enable: u8,
        mTimerSourceClockType: BL_MTimer_Source_Clock_Type,
        div: u16,
    ) -> *const i32 = mem::transmute(ROM_DRIVER_TABLE.unwrap()[24]);

    GLB_Power_On_XTAL_And_PLL_CLK(
        XTALType::GLB_XTAL_40M,
        PLLType::GLB_PLL_WIFIPLL as i32 | PLLType::GLB_PLL_AUPLL as i32,
    );
    GLB_Set_MCU_System_CLK(GLB_MCU_SYS_CLK_Type::GLB_MCU_SYS_CLK_TOP_WIFIPLL_320M);

    // TODO
    // CPU_Set_MTimer_CLK(true, BL_MTimer_Source_Clock_Type::BL_MTIMER_SOURCE_CLOCK_MCU_XCLK,
}

#[repr(C)]
pub enum HBN_UART_CLK_Type {
    HBN_UART_CLK_MCU_BCLK = 0, /* Select mcu_pbclk as UART clock */
    HBN_UART_CLK_MUXPLL_160M,  /* Select MUXPLL 160M as UART clock */
    HBN_UART_CLK_XCLK,         /* Select XCLK as UART clock */
}

unsafe fn peripheral_clock_init() {
    /*
    #define PERIPHERAL_CLOCK_UART0_ENABLE()                           \
    do {                                                          \
        volatile uint32_t regval = getreg32(BFLB_GLB_CGEN1_BASE); \
        regval |= (1 << 16);                                      \
        putreg32(regval, BFLB_GLB_CGEN1_BASE);                    \
    } while (0)
     */
    let reg: *mut u32 = mem::transmute(0x20000000 + 0x584);
    ptr::write_volatile(reg, ptr::read_volatile(reg) | (1 << 16));
    // PERIPHERAL_CLOCK_UART1_ENABLE
    ptr::write_volatile(reg, ptr::read_volatile(reg) | (1 << 17));

    let GLB_Set_UART_CLK: unsafe extern "C" fn(
        enable: u8,
        clkSel: HBN_UART_CLK_Type,
        div: u8,
    ) -> *const i32 = mem::transmute(ROM_DRIVER_TABLE.unwrap()[161]);
    let GLB_Set_USB_CLK_From_WIFIPLL: unsafe extern "C" fn(enable: u8) -> *const i32 =
        mem::transmute(ROM_DRIVER_TABLE.unwrap()[162]);

    GLB_Set_UART_CLK(true as u8, HBN_UART_CLK_Type::HBN_UART_CLK_XCLK, 0);
}

unsafe fn bflb_gpio_init() {
    let reg = &(*pac::GLB::ptr());

    reg.gpio_config[27].modify(|_, w| {
        w.pin_mode()
            .output_value()
            .pull_up()
            .set_bit()
            .schmitt()
            .set_bit()
            .drive()
            .bits(0)
    });
}

#[riscv_rt::entry]
unsafe fn main() -> ! {
    unsafe {
        let start_addr: u32 = 0x90015800;
        ROM_DRIVER_TABLE = Some(mem::transmute(start_addr));
    }

    //system_clock_init();
    //peripheral_clock_init();

    // # board_init
    // bflb_flash_init
    // system_clock_init
    // peripheral_clock_init

    // bflb_irq_initialize?
    // console_init
    // kmem_init

    // bflb_gpio_init
    bflb_gpio_init();
    // bflb_gpio_set
    let reg = &(*pac::GLB::ptr());
    loop {
        reg.gpio_set[0].modify(|_, w| w.bits(1 << 27));
        riscv::asm::delay(8_000_000);
        reg.gpio_clear[0].modify(|_, w| w.bits(1 << 27));
        riscv::asm::delay(8_000_000);
    }
}
