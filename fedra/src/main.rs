#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use cortex_m::peripheral::{syst::SystClkSource, DWT};
use cortex_m_rt::{exception, ExceptionFrame};
use cortex_m_semihosting::{debug, hprint, hprintln};
use hal::prelude::*;
use panic_semihosting as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32f4::stm32f429::{Interrupt, Peripherals, NVIC};
use stm32f4xx_hal as hal;

const PERIOD: u32 = 8_000_000;

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT
)]
const APP: () = {
    struct Resources {
        cp: rtic::Peripherals,
        dp: hal::stm32::Peripherals,
    }

    #[init(schedule = [trigger])]
    fn init(cx: init::Context) -> init::LateResources {
        hprintln!("Hello world").unwrap();

        let mut cp: rtic::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;

        cp.DCB.enable_trace();
        DWT::unlock();
        cp.DWT.enable_cycle_counter();

        let now = cx.start;
        hprintln!("init @ {:?}", now).unwrap();

        cx.schedule.trigger(now + PERIOD.cycles()).unwrap();

        // let rcc = dp.RCC.constrain();
        // let mut syst = cp.SYST;
        // setup_clocks(rcc, &mut syst);

        init::LateResources { cp, dp }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("Idle").unwrap();

        loop {}
    }

    #[task(schedule = [trigger])]
    fn trigger(cx: trigger::Context) {
        // hprintln!("trigger @ {:?}", Instant::now()).unwrap();
        rtic::pend(Interrupt::EXTI0);
        cx.schedule.trigger(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(binds = EXTI0)]
    fn exti0(_: exti0::Context) {
        hprint!(".").unwrap();
    }

    extern "C" {
        fn UART4();
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
