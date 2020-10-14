#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]

pub mod qemu;

use core::panic::PanicInfo;
use cortex_m::peripheral::DWT;
use cortex_m_rt::{exception, ExceptionFrame};
use hal::prelude::*;
use rtic::cyccnt::U32Ext;
use rtt_target::{rprint, rprintln, rtt_init_print};
use stm32f4::stm32f429::Interrupt;
use stm32f4xx_hal as hal;

const PERIOD: u32 = 180_000_000;

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT
)]
const APP: () = {
    struct Resources {
        cp: rtic::Peripherals,
    }

    #[init(schedule = [trigger])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();

        rprintln!("Hello world");

        let mut cp: rtic::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;

        let rcc = dp.RCC.constrain();
        setup_clocks(rcc);

        cp.DCB.enable_trace();
        DWT::unlock();
        cp.DWT.enable_cycle_counter();

        let now = cx.start;
        rprintln!("init @ {:?}", now);

        cx.schedule.trigger(now + PERIOD.cycles()).unwrap();

        init::LateResources { cp }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("Idle");

        loop {}
    }

    #[task(schedule = [trigger])]
    fn trigger(cx: trigger::Context) {
        // rprintln!("trigger @ {:?}", Instant::now());
        rtic::pend(Interrupt::EXTI0);
        cx.schedule.trigger(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(binds = EXTI0)]
    fn exti0(_: exti0::Context) {
        rprint!(".\n");
    }

    extern "C" {
        fn EXTI1();
    }
};

#[allow(unused)]
fn setup_clocks(rcc: hal::rcc::Rcc) {
    rcc.cfgr.sysclk(180.mhz()).freeze();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    nop_loop()
}

#[exception]
fn DefaultHandler(irqn: i16) {
    rprintln!("Exception IRQN {}", irqn);

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    rprintln!("HardFault {:#?}", ef);

    loop {}
}

#[allow(unused)]
fn nop_loop() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
