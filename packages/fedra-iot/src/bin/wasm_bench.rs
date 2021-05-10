#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    dispatchers = [EXTI1]
)]
mod app {
    use fedra_iot::wasm_host;
    use fedra_iot::{memory, time};
    use hal::prelude::*;
    use stm32f4xx_hal as hal;
    use wasmi::{ImportsBuilder, NopExternals, StackRecycler};

    const VALUE_STACK_LIMIT: usize = 4 * 1024;
    const CALL_STACK_LIMIT: usize = 42;

    #[resources]
    struct Resources {}

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        fedra_iot::logger::init_logger();
        memory::setup_heap();

        let dp: hal::stm32::Peripherals = cx.device;
        let rcc = dp.RCC.constrain();
        let tim2 = dp.TIM2;

        let _clocks = time::setup_clocks(rcc, tim2);

        run::spawn().unwrap();

        (init::LateResources {}, init::Monotonics())
    }

    #[idle(resources = [])]
    fn idle(_cx: idle::Context) -> ! {
        super::nop_loop();
    }

    #[task]
    fn run(_cx: run::Context) {
        // log::info!("{}", core::mem::size_of::<Instruction>());
        let stack_recycler = StackRecycler::with_limits(VALUE_STACK_LIMIT, CALL_STACK_LIMIT);

        let mut host = wasm_host::WasmHost {
            imports: ImportsBuilder::default(),
            instance: None,
            stack_recycler,
        };

        let wasm = include_bytes!("./wasm/trmm.wasm");
        memory::stats();
        let module = wasm_host::WasmHost::create_module(wasm).unwrap();
        memory::stats();
        host.set_instance(&module).unwrap();
        memory::stats();

        let mut times = [0_u64; 15];
        let instance = host.instance.as_ref().unwrap();

        for i in 0..15 {
            host.stack_recycler.clear();
            let now = time::now();
            instance
                .invoke_export_with_stack("bench", &[], &mut NopExternals, &mut host.stack_recycler)
                .unwrap();
            let elapsed = time::now() - now;
            times[i] = elapsed;
        }

        let average = times.iter().fold(0, |acc, x| acc + x) / 15;
        log::info!("average execution time: {}, {:?}", average, times);
    }

    #[task(binds = TIM2, priority = 14)]
    fn tim2(_: tim2::Context) {
        time::handle_tim2_interrupt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    nop_loop()
}

#[exception]
fn DefaultHandler(irqn: i16) {
    log::error!("Exception IRQN {}", irqn);

    nop_loop()
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    log::error!("HardFault {:#?}", ef);

    nop_loop()
}

fn nop_loop() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
