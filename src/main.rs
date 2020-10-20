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

#[allow(unused)]
const PERIOD: u32 = 180_000_000;

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT
)]
const APP: () = {
    struct Resources {
        env: i32,
    }

    #[init(schedule = [trigger])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();

        let mut cp: rtic::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;

        setup_clocks(&mut cp, dp);

        cx.schedule.trigger(cx.start + PERIOD.cycles()).unwrap();
        init::LateResources { env: 0 }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("Idle");
        rtic::pend(Interrupt::EXTI0);

        loop {}
    }

    #[task(schedule = [trigger])]
    fn trigger(cx: trigger::Context) {
        rtic::pend(Interrupt::EXTI0);
        cx.schedule.trigger(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(binds = EXTI0, resources = [])]
    fn exti0(_: exti0::Context) {
        rprint!(".");
    }

    extern "C" {
        fn EXTI1();
    }
};

fn setup_clocks(cp: &mut rtic::Peripherals, dp: hal::stm32::Peripherals) {
    let rcc = dp.RCC.constrain();

    rcc.cfgr.sysclk(180.mhz()).freeze();
    cp.DCB.enable_trace();
    DWT::unlock();
    cp.DWT.enable_cycle_counter();
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
