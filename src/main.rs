#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod allocator;
mod intrinsics;
mod network;
pub mod qemu;

use core::cell::RefCell;
use core::panic::PanicInfo;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::SYST;
use cortex_m_rt::{exception, ExceptionFrame};
use hal::prelude::*;
use rtt_target::rprintln;

use stm32f4xx_hal as hal;

#[allow(unused)]
const PERIOD: u32 = 168_000_000;

static TIME: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    dispatchers = [EXTI1]
)]
mod app {
    use super::network;
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
        super::setup_heap();

        let mut cp: cortex_m::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;
        let rcc = dp.RCC.constrain();

        let clocks = super::setup_clocks(&mut cp, rcc);

        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();
        let ethernet_mac = dp.ETHERNET_MAC;
        let ethernet_dma = dp.ETHERNET_DMA;

        rprintln!("HCLK {}", clocks.hclk().0);
        rprintln!("SYSCLK {}", clocks.sysclk().0);

        network::setup_eth(gpioa, gpiob, gpioc, clocks, ethernet_mac, ethernet_dma);
        network::setup_iface();
        network::create_sockets();

        init::LateResources { noop: 0 }
    }

    #[idle(resources = [])]
    fn idle(_: idle::Context) -> ! {
        loop {}
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

    #[task(resources = [])]
    fn server(_: server::Context) {
        network::handle_request();
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

fn setup_clocks(cp: &mut cortex_m::Peripherals, rcc: hal::rcc::Rcc) -> hal::rcc::Clocks {
    let syst = &mut cp.SYST;

    // HCLK 42 so that the resulting value is >=24Mhz after division
    let clocks = rcc.cfgr.sysclk(168.mhz()).hclk(42.mhz()).freeze();

    syst.set_reload(SYST::get_ticks_per_10ms() / 10);
    syst.enable_counter();
    syst.enable_interrupt();

    clocks
}

fn setup_heap() {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024 * (128 / 2); // Reserve half RAM

    unsafe {
        allocator::ALLOCATOR.init(start, size);
    }
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        let mut time = TIME.borrow(cs).borrow_mut();
        *time += 1;
    })
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
