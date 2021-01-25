#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod coap_server;
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
    use super::{coap_server, memory, network, time, wasm_host};
    use alloc::string::ToString;
    use coap_lite::{ContentFormat, RequestType as Method, ResponseType as Status};
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

        coap_server: coap_server::CoapServer,

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

        let coap_server = coap_server::CoapServer::default();

        let mut host = wasm_host::WasmHost::default();
        let runtime = wasm_host::Runtime::default();
        host.setup_default().expect("Could not setup the WAST Host");

        temp::schedule(cx.start + ACTIVATION_OFFSET.cycles()).unwrap();

        init::LateResources {
            host,
            runtime,
            coap_server,
        }
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

            notify::spawn("sensors/temp", runtime.temp).unwrap();
        });
    }

    #[task(resources = [coap_server], capacity = 2, priority = 1)]
    fn notify(cx: notify::Context, resource: &'static str, value: i32) {
        let mut coap_server = cx.resources.coap_server;
        let payload = value.to_string().into_bytes();

        coap_server.lock(|coap_server: &mut coap_server::CoapServer| {
            coap_server.notify_observers(resource, payload);
        });

        rprintln!("Notified");
    }

    #[task(resources = [runtime, coap_server], priority = 1)]
    fn server(cx: server::Context) {
        let mut runtime = cx.resources.runtime;
        let mut coap_server = cx.resources.coap_server;

        coap_server.lock(|coap_server: &mut coap_server::CoapServer| {
            coap_server.handle_request(|request| {
                let method = request.get_method().clone();
                let path = request.get_path();
                let mut response = request.response?;

                rprintln!("Request path {}", path);

                match (method, path.as_str()) {
                    (Method::Get, "sensors/temp") => {
                        runtime.lock(|runtime| {
                            let temp = runtime.temp.to_string();
                            response.message.payload = temp.into_bytes();
                        });
                    }
                    (Method::Get, "well-known/core") => {
                        response
                            .message
                            .set_content_format(ContentFormat::ApplicationLinkFormat);
                        response.message.payload =
                            br#"</sensors/temp>;rt="oic.r.temperature";if="sensor""#.to_vec();
                    }
                    (Method::Get, "ping") => {
                        response.message.payload = b"pong".to_vec();
                    }
                    _ => {
                        response.set_status(Status::NotFound);
                        response.message.payload = b"Not found".to_vec();
                    }
                }

                Some(response)
            });
        })
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
