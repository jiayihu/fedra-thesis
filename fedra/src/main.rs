#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod intrinsics;
mod memory;
mod network;
pub mod qemu;
mod time;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};
use rtt_target::rprintln;

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    dispatchers = [EXTI1]
)]
mod app {
    use super::{memory, network, time};
    use hal::prelude::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal as hal;
    use wasmi::{ImportsBuilder, ModuleInstance, NopExternals, RuntimeValue};

    #[resources]
    struct Resources {
        noop: u32,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        memory::setup_heap();

        let mut cp: cortex_m::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;
        let rcc = dp.RCC.constrain();

        let clocks = time::setup_clocks(&mut cp, rcc);

        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();
        let ethernet_mac = dp.ETHERNET_MAC;
        let ethernet_dma = dp.ETHERNET_DMA;

        network::setup_eth(gpioa, gpiob, gpioc, clocks, ethernet_mac, ethernet_dma);
        network::setup_net();
        network::create_sockets();

        init::LateResources { noop: 0 }
    }

    #[idle(resources = [])]
    fn idle(_: idle::Context) -> ! {
        super::nop_loop();
    }

    #[task]
    fn wasm(_: wasm::Context) {
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
    }

    #[task(resources = [], priority = 1)]
    fn server(_: server::Context) {
        network::handle_request(|request| {
            let path = request.get_path();
            let mut response = request.response?;

            rprintln!("Request path {}", path);

            match path.as_str() {
                "sensors/temp" => {
                    wasm::spawn().unwrap();
                }
                "well-known/core" => {
                    response
                        .message
                        .set_content_format(ContentFormat::ApplicationLinkFormat);
                    response.message.set_payload(
                        b"\
                    </sensors/temp>;rt=\"oic.r.temperature\";if=\"sensor\"
                    "
                        .to_vec(),
                    );
                }
                "ping" => {
                    response.message.set_payload(b"pong".to_vec());
                }
                _ => {
                    response.message.header.code = MessageClass::Response(ResponseType::NotFound);
                    response.message.set_payload(b"Not found".to_vec());
                }
            }

            Some(response)
        });
    }

    #[task(binds = ETH, resources = [])]
    fn eth(_: eth::Context) {
        network::set_pending();

        // Clear interrupt flags
        let p = unsafe { hal::stm32::Peripherals::steal() };
        stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);

        server::spawn().unwrap();
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

fn nop_loop() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
