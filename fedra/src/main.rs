#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{exception, ExceptionFrame};
use cortex_m_semihosting::{debug, hprint, hprintln};
use hal::prelude::*;
use panic_semihosting as _;
use stm32f4::stm32f429::{Interrupt, NVIC};
use stm32f4xx_hal as hal;

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        hprintln!("Hello world").unwrap();

        let cp: cortex_m::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;

        let rcc = dp.RCC.constrain();
        let mut syst = cp.SYST;

        setup_clocks(rcc, &mut syst);

        unsafe {
            NVIC::unmask(Interrupt::EXTI0);
        }

        rtic::pend(Interrupt::EXTI0);
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("Idle").unwrap();
        rtic::pend(Interrupt::EXTI0);

        loop {}
    }

    #[task(binds = EXTI0)]
    fn exti0(_: exti0::Context) {
        hprint!(".").unwrap();
    }
};

#[allow(unused)]
fn setup_clocks(rcc: hal::rcc::Rcc, syst: &mut cortex_m::peripheral::SYST) {
    rcc.cfgr.sysclk(180.mhz()).freeze();

    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(180_000_000);
    syst.clear_current();
    syst.enable_counter();
}

#[exception]
fn DefaultHandler(irqn: i16) {
    hprint!("Exception IRQN {}", irqn).unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    hprint!("HardFault {:#?}", ef).unwrap();

    loop {}
}

#[allow(unused)]
fn nop_loop() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}

#[allow(unused)]
fn exit_qemu() {
    debug::exit(debug::EXIT_SUCCESS);
}
