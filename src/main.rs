#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod allocator;
mod intrinsics;
pub mod qemu;

use core::panic::PanicInfo;
use cortex_m::peripheral::DWT;
use cortex_m_rt::{exception, ExceptionFrame};
use hal::prelude::*;
use rtt_target::{rprint, rprintln, rtt_init_print};
use stm32f4xx_hal as hal;

use wasmi::{ImportsBuilder, ModuleInstance, NopExternals, RuntimeValue};

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

    #[init(schedule = [])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();

        let mut cp: cortex_m::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;

        setup_clocks(&mut cp, dp);

        setup_heap();

        init::LateResources { env: 0 }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let wasm = include_bytes!("./wasm/add.wasm");
        let module = wasmi::Module::from_buffer(&wasm).expect("Failed to load wasm");
        assert!(module.deny_floating_point().is_ok());

        let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
            .expect("Failed to instantiate wasm module");
        let instance = instance
            .run_start(&mut NopExternals)
            .expect("WASM start function trapped");

        let result = instance
            .invoke_export(
                "add",
                &[RuntimeValue::I32(1), RuntimeValue::I32(2)],
                &mut NopExternals,
            )
            .expect("Failed to execute export");

        match result {
            Some(RuntimeValue::I32(x)) => rprintln!("1 + 2 = {}", x),
            Some(_) => rprintln!("Unexpected result"),
            None => rprintln!("No result"),
        }

        loop {}
    }

    #[task(binds = EXTI0, resources = [])]
    fn exti0(_: exti0::Context) {
        rprint!(".");
    }

    extern "C" {
        fn EXTI1();
    }
};

fn setup_clocks(cp: &mut cortex_m::Peripherals, dp: hal::stm32::Peripherals) {
    let rcc = dp.RCC.constrain();

    rcc.cfgr.sysclk(180.mhz()).freeze();
    cp.DCB.enable_trace();
    DWT::unlock();
    cp.DWT.enable_cycle_counter();
}

fn setup_heap() {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024 * (192 / 2); // Reserve half RAM

    unsafe {
        allocator::ALLOCATOR.init(start, size);
    }
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
