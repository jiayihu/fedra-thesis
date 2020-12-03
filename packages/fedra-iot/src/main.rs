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
mod wasm_host;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};
use rtt_target::rprintln;

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
    dispatchers = [EXTI1, EXTI2]
)]
mod app {
    use super::{memory, network, time, wasm_host};
    use alloc::string::ToString;
    use coap_lite::{ContentFormat, MessageClass, ResponseType};
    use hal::prelude::*;
    use rtic::cyccnt::U32Ext;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal as hal;

    const PERIOD: u32 = 160_000_000;

    // Relative offset of task activation after initial elaboration
    const ACTIVATION_OFFSET: u32 = PERIOD;

    #[resources]
    struct Resources {
        #[task_local]
        host: wasm_host::WasmHost<'static>,

        runtime: wasm_host::Runtime,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        memory::setup_heap();

        let mut cp: rtic::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;
        let rcc = dp.RCC.constrain();
        let tim2 = dp.TIM2;

        time::setup_cycle_counter(&mut cp);
        let clocks = time::setup_clocks(rcc, tim2);

        let eth_p = network::EthPeripherals {
            gpioa: dp.GPIOA.split(),
            gpiob: dp.GPIOB.split(),
            gpioc: dp.GPIOC.split(),
            ethernet_mac: dp.ETHERNET_MAC,
            ethernet_dma: dp.ETHERNET_DMA,
        };

        network::setup_eth(eth_p, clocks);
        network::setup_net();
        network::create_sockets();

        let mut host = wasm_host::WasmHost::default();
        let runtime = wasm_host::Runtime::default();
        host.setup_default().expect("Could not setup the WAST Host");

        temp::schedule(cx.start + ACTIVATION_OFFSET.cycles()).unwrap();

        init::LateResources { host, runtime }
    }

    #[idle(resources = [])]
    fn idle(_cx: idle::Context) -> ! {
        super::nop_loop();
    }

    #[task(resources = [host, runtime], priority = 2)]
    fn temp(cx: temp::Context) {
        temp::schedule(cx.scheduled + (PERIOD * 5).cycles()).unwrap();

        let host = cx.resources.host;
        let mut runtime = cx.resources.runtime;

        runtime.lock(|runtime: &mut wasm_host::Runtime| {
            host.invoke("main", runtime)
                .expect("Cannot invoke main in the WASM module"); // TODO: Handle as CoAP response

            rprintln!("Temp {}", runtime.temp);
        })
    }

    #[task(resources = [runtime], priority = 1)]
    fn server(cx: server::Context) {
        let mut runtime = cx.resources.runtime;

        network::handle_request(|request| {
            let path = request.get_path();
            let mut response = request.response?;

            rprintln!("Request path {}", path);

            match path.as_str() {
                "sensors/temp" => {
                    runtime.lock(|runtime| {
                        let temp = runtime.temp.to_string();
                        response.message.payload = temp.into_bytes();
                    });
                }
                "well-known/core" => {
                    response
                        .message
                        .set_content_format(ContentFormat::ApplicationLinkFormat);
                    response.message.payload = b"\
                    </sensors/temp>;rt=\"oic.r.temperature\";if=\"sensor\"
                    "
                    .to_vec();
                }
                "ping" => {
                    response.message.payload = b"pong".to_vec();
                }
                _ => {
                    response.message.header.code = MessageClass::Response(ResponseType::NotFound);
                    response.message.payload = b"Not found".to_vec();
                }
            }

            Some(response)
        });
    }

    #[task(binds = ETH, resources = [], priority = 10)]
    fn eth(_: eth::Context) {
        network::set_pending();

        // Clear interrupt flags
        let p = unsafe { hal::stm32::Peripherals::steal() };
        stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);

        server::spawn().unwrap();
    }

    #[task(binds = TIM2, priority = 14)]
    fn tim2(_: tim2::Context) {
        time::handle_tim2_interrupt();
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

    nop_loop()
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    rprintln!("HardFault {:#?}", ef);

    nop_loop()
}

fn nop_loop() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
