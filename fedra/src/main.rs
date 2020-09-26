#![no_std]
#![no_main]

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprint};
use hal::prelude::*;
use panic_semihosting as _;
use stm32f4::stm32f429::{interrupt, Interrupt, NVIC};
use stm32f4xx_hal as hal;

#[entry]
fn main() -> ! {
    hprint!("Hello world").unwrap();

    let cortex_peri = cortex_m::Peripherals::take().unwrap();
    let device_peri = hal::stm32::Peripherals::take().unwrap();

    let rcc = device_peri.RCC.constrain();
    setup_clocks(rcc);

    let mut syst = cortex_peri.SYST;
    let mut nvic = cortex_peri.NVIC;

    nvic.enable(Interrupt::EXTI0);

    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(180_000_000);
    syst.clear_current();
    syst.enable_counter();

    // exit_qemu();
    loop {
        while !syst.has_wrapped() {}

        NVIC::pend(Interrupt::EXTI0);
    }
}

#[interrupt]
fn EXTI0() {
    hprint!(".").unwrap();
}

fn setup_clocks(rcc: hal::rcc::Rcc) {
    rcc.cfgr.sysclk(180.mhz()).freeze();
}

fn nop_loop() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}

#[allow(unused)]
fn exit_qemu() {
    debug::exit(debug::EXIT_SUCCESS);
}
