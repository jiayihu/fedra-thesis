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
    dispatchers = [EXTI1, EXTI2, EXTI3, EXTI4]
)]
mod app {
    use alloc::string::ToString;
    use coap_lite::{ContentFormat, RequestType as Method, ResponseType as Status};
    use dwt_systick_monotonic::DwtSystick;
    use fedra_iot::{coap_server, memory, network, time, wasm_host};
    use hal::prelude::*;
    use rtic::time::duration::Seconds;
    use stm32f4xx_hal as hal;

    const PERIOD: u32 = 160_000_000;

    // Relative offset of task activation after initial elaboration
    const ACTIVATION_OFFSET: u32 = 1; //1s

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<160_000_000>;

    #[resources]
    struct Resources {
        coap_server: coap_server::CoapServer,

        runtime: wasm_host::Runtime,
    }

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        fedra_iot::logger::init_logger();
        memory::setup_heap();

        let mut cp: rtic::export::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;
        let rcc = dp.RCC.constrain();
        let tim2 = dp.TIM2;

        let mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, PERIOD);
        let clocks = time::setup_clocks(rcc, tim2);

        let eth_p = network::EthPeripherals {
            gpioa: dp.GPIOA.split(),
            gpiob: dp.GPIOB.split(),
            gpioc: dp.GPIOC.split(),
            ethernet_mac: dp.ETHERNET_MAC,
            ethernet_dma: dp.ETHERNET_DMA,
        };

        network::setup_eth(eth_p, clocks.clone());
        network::setup_net();
        network::create_sockets();

        let coap_server = coap_server::CoapServer::default();

        let runtime = wasm_host::Runtime::default();

        rainfall::spawn_after(Seconds(ACTIVATION_OFFSET)).unwrap();

        (
            init::LateResources {
                runtime,
                coap_server,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(resources = [])]
    fn idle(_cx: idle::Context) -> ! {
        super::nop_loop();
    }

    #[task(resources = [runtime], priority = 5)]
    fn rainfall(cx: rainfall::Context) {
        static mut RAINFALL: f32 = 0_f32;

        rainfall::spawn_after(Seconds(5_u32)).unwrap();

        let mut runtime = cx.resources.runtime;

        runtime.lock(|runtime: &mut wasm_host::Runtime| {
            // let rainfall = sample::gen_rainfall(rand_source);
            *RAINFALL = *RAINFALL + 1_f32;
            runtime.rainfall = *RAINFALL;

            log::info!("Rainfall {}", runtime.rainfall);

            if notify::spawn("sensors/rainfall", runtime.rainfall).is_err() {
                log::info!("Can't spawn notify, skipping");
            }
        });
    }

    #[task(resources = [coap_server], capacity = 1, priority = 1)]
    fn notify(cx: notify::Context, resource: &'static str, value: f32) {
        let mut coap_server = cx.resources.coap_server;
        let payload = value.to_string().into_bytes();

        coap_server.lock(|coap_server: &mut coap_server::CoapServer| {
            coap_server.notify_observers(resource, payload);
        });
    }

    #[task(resources = [runtime, coap_server], capacity = 3, priority = 1)]
    fn server(cx: server::Context) {
        let mut runtime = cx.resources.runtime;
        let mut coap_server = cx.resources.coap_server;

        coap_server.lock(|coap_server: &mut coap_server::CoapServer| {
            coap_server.handle_request(|request| {
                let method = request.get_method().clone();
                let path = request.get_path();
                let mut response = request.response?;

                log::info!("Request path {}", path);

                match (method, path.as_str()) {
                    (Method::Get, "sensors/rainfall") => {
                        runtime.lock(|runtime| {
                            let rainfall = runtime.rainfall.to_string();
                            response.message.payload = rainfall.into_bytes();
                        });
                    }
                    (Method::Get, "well-known/core") => {
                        response
                            .message
                            .set_content_format(ContentFormat::ApplicationLinkFormat);
                        response.message.payload =
                            br#"</sensors/rainfall>;rt="rainfall";if="sensor",
                            </sensors/flow>;rt="flow";if="sensor""#
                                .to_vec();
                    }
                    (Method::Get, "ping") => {
                        response.message.payload = b"pong".to_vec();
                    }
                    _ => {
                        response.set_status(Status::NotFound);
                        response.message.payload = b"Not found".to_vec();
                    }
                }

                response
                    .message
                    .set_content_format(ContentFormat::TextPlain);

                Some(response)
            });
        })
    }

    #[task(capacity = 2, priority = 1)]
    fn socket(_: socket::Context) {
        let ready = network::check_socket_readiness();

        if ready {
            server::spawn().unwrap();
        }
    }

    #[task(binds = ETH, resources = [], priority = 10)]
    fn eth(_: eth::Context) {
        network::set_pending();

        // Clear interrupt flags
        let p = unsafe { hal::stm32::Peripherals::steal() };
        stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);

        socket::spawn().unwrap();
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
