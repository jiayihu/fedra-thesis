#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod allocator;
pub mod intrinsics;
pub mod qemu;

use core::panic::PanicInfo;
use cortex_m::peripheral::DWT;
use cortex_m_rt::{exception, ExceptionFrame};
use hal::prelude::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4::stm32f429::Interrupt;
use stm32f4xx_hal as hal;
use wasm3::{Environment, Module};

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

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        setup_heap();
        rtt_init_print!();

        let mut cp: cortex_m::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;

        setup_clocks(&mut cp, dp);

        init::LateResources { env: 0 }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rtic::pend(Interrupt::EXTI0);

        let env = Environment::new().expect("Unable to create environment");

        let runtime = env
            .create_runtime(1024 * 60)
            .expect("Unable to create runtime");
        let wasm = Module::parse(&env, &include_bytes!("wasm/add.wasm")[..])
            .expect("Unable to parse module");

        let module = runtime.load_module(wasm).expect("Unable to load module");
        let func = module
            .find_function::<(i64, i64), i64>("add")
            .expect("Unable to find function");
        let result = func.call(3, 6).unwrap();

        rprintln!("Wasm says that 3 + 6 is {}", result);

        loop {}
    }

    #[task(binds = EXTI0, resources = [env])]
    fn exti0(_: exti0::Context) {
        rprintln!("Calling add WASM");
    }

    extern "C" {
        fn EXTI1();
    }
};

fn setup_heap() {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024 * 20; // bytes

    unsafe {
        allocator::ALLOCATOR.init(start, size);

        rprintln!(
            "Size of Rc {}",
            core::intrinsics::size_of::<alloc::rc::Rc<i32>>()
        )
    }
}

#[allow(unused)]
fn setup_clocks(cp: &mut cortex_m::Peripherals, dp: hal::stm32::Peripherals) {
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
