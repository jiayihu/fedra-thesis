#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod allocator;
mod intrinsics;
pub mod qemu;

use core::cell::RefCell;
use core::fmt::Write;
use core::panic::PanicInfo;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::SYST;
use cortex_m_rt::{exception, ExceptionFrame};
use hal::prelude::*;
use once_cell::unsync::OnceCell;
use rtt_target::{rprintln, rtt_init_print};
use smoltcp::iface::{EthernetInterface, EthernetInterfaceBuilder, NeighborCache};
use smoltcp::socket::{SocketHandle, SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
use smoltcp::Error;
use stm32_eth::{
    Eth, EthPins, PhyAddress, RingEntry, RxDescriptor, RxRingEntry, TxDescriptor, TxRingEntry,
};
use stm32f4xx_hal as hal;

#[allow(unused)]
const PERIOD: u32 = 180_000_000;

static TIME: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));
static ETH_PENDING: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

#[rtic::app(
    device = stm32f4xx_hal::stm32,
    peripherals = true,
    dispatchers = [EXTI1]
)]
mod APP {
    struct Resources {
        iface: EthernetInterface<'static, 'static, 'static, &'static mut Eth<'static, 'static>>,
        sockets: SocketSet<'static, 'static, 'static>,
        server_handle: SocketHandle,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        static mut rx_ring_cell: OnceCell<[RingEntry<RxDescriptor>; 8]> = OnceCell::new();
        static mut tx_ring_cell: OnceCell<[RingEntry<TxDescriptor>; 2]> = OnceCell::new();
        static mut eth_cell: OnceCell<Eth<'static, 'static>> = OnceCell::new();
        static mut iface_cell: OnceCell<
            EthernetInterface<'static, 'static, 'static, &'static mut Eth<'static, 'static>>,
        > = OnceCell::new();
        static mut server_rx_buffer: [u8; 2048] = [0; 2048];
        static mut server_tx_buffer: [u8; 2048] = [0; 2048];
        static mut sockets_cell: OnceCell<SocketSet<'static, 'static, 'static>> = OnceCell::new();
        static mut server_handle_cell: OnceCell<SocketHandle> = OnceCell::new();

        rtt_init_print!();
        setup_heap();

        let mut cp: cortex_m::Peripherals = cx.core;
        let dp: hal::stm32::Peripherals = cx.device;
        let rcc = dp.RCC.constrain();

        let clocks = setup_clocks(&mut cp, rcc);

        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();
        let ethernet_mac = dp.ETHERNET_MAC;
        let ethernet_dma = dp.ETHERNET_DMA;

        let rx_ring: &'static mut [RingEntry<RxDescriptor>; 8] =
            rx_ring_cell.get_or_init(|| Default::default());
        let tx_ring: &'static mut [RingEntry<TxDescriptor>; 2] =
            tx_ring_cell.get_or_init(|| Default::default());

        let eth = eth_cell.get_or_init(|| {
            let mut eth = setup_ethernet(
                gpioa,
                gpiob,
                gpioc,
                ethernet_mac,
                ethernet_dma,
                clocks,
                rx_ring,
                tx_ring,
            );

            eth
        });

        let iface = iface_cell.get_or_init(|| {
            const SRC_MAC: [u8; 6] = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
            let local_addr = Ipv4Address::new(192, 168, 1, 100);
            let ip_addr = IpCidr::new(IpAddress::from(local_addr), 24);
            let mut ip_addrs = [ip_addr];
            let mut neighbor_storage = [None; 16];
            let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
            let ethernet_addr = EthernetAddress(SRC_MAC);
            let iface = EthernetInterfaceBuilder::new(&mut *eth)
                .ethernet_addr(ethernet_addr)
                .ip_addrs(&mut ip_addrs[..])
                .neighbor_cache(neighbor_cache)
                .finalize();

            iface
        });

        let sockets = sockets_cell.get_or_init(|| {
            let mut sockets_storage = [None, None];
            let mut sockets = SocketSet::new(&mut sockets_storage[..]);

            sockets
        });

        let server_handle = server_handle_cell.get_or_init(|| {
            let server_socket = TcpSocket::new(
                TcpSocketBuffer::new(&mut server_rx_buffer[..]),
                TcpSocketBuffer::new(&mut server_tx_buffer[..]),
            );
            let server_handle = sockets.add(server_socket);

            rprintln!("Listening at :80");

            server_handle
        });

        init::LateResources {
            iface: *iface,
            sockets: *sockets,
            server_handle: *server_handle,
        }
    }

    #[idle(resources = [])]
    fn idle(cx: idle::Context) -> ! {
        loop {}
    }

    #[task(resources = [iface, sockets, server_handle])]
    fn server(cx: server::Context) {
        let iface = cx.resources.iface;
        let sockets = cx.resources.sockets;
        let server_handle = cx.resources.server_handle;

        let time: u64 = cortex_m::interrupt::free(|cs| *TIME.borrow(cs).borrow());

        cortex_m::interrupt::free(|cs| {
            let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
            *eth_pending = false;
        });

        match iface.poll(&mut sockets, Instant::from_millis(time as i64)) {
            Ok(true) => {
                let mut socket = sockets.get::<TcpSocket>(*server_handle);
                if !socket.is_open() {
                    socket
                        .listen(80)
                        .unwrap_or_else(|e| rprintln!("TCP listen error: {:?}", e));
                }

                if socket.can_send() {
                    write!(socket, "hello\n")
                        .map(|_| {
                            socket.close();
                        })
                        .unwrap_or_else(|e| rprintln!("TCP send error: {:?}", e));
                }
            }
            Ok(false) => {
                // Sleep task if no ethernet work is pending
            }
            Err(e) => {
                // Ignore malformed packets, they are pretty common
                // if e != Error::Unrecognized {
                rprintln!("Error: {:?}", e);
                // }
            }
        }
    }

    #[task(binds = ETH, resources = [])]
    fn eth(_: eth::Context) {
        cortex_m::interrupt::free(|cs| {
            let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
            *eth_pending = true;
        });

        // Clear interrupt flags
        let p = unsafe { hal::stm32::Peripherals::steal() };
        stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);
    }
}

fn setup_clocks(cp: &mut cortex_m::Peripherals, rcc: hal::rcc::Rcc) -> hal::rcc::Clocks {
    let syst = &mut cp.SYST;

    let clocks = rcc.cfgr.sysclk(168.mhz()).hclk(32.mhz()).freeze();

    syst.set_reload(SYST::get_ticks_per_10ms() / 10);
    syst.enable_counter();
    syst.enable_interrupt();

    return clocks;
}

fn setup_ethernet<'rx, 'tx>(
    gpioa: hal::gpio::gpioa::Parts,
    gpiob: hal::gpio::gpiob::Parts,
    gpioc: hal::gpio::gpioc::Parts,
    ethernet_mac: hal::stm32::ETHERNET_MAC,
    ethernet_dma: hal::stm32::ETHERNET_DMA,
    clocks: hal::rcc::Clocks,
    rx_ring: &'rx mut [RxRingEntry],
    tx_ring: &'tx mut [TxRingEntry],
) -> Eth<'rx, 'tx> {
    rprintln!("Enabling ethernet...");

    let eth_pins = EthPins {
        ref_clk: gpioa.pa1,
        md_io: gpioa.pa2,
        md_clk: gpioc.pc1,
        crs: gpioa.pa7,
        tx_en: gpiob.pb11,
        tx_d0: gpiob.pb12,
        tx_d1: gpiob.pb13,
        rx_d0: gpioc.pc4,
        rx_d1: gpioc.pc5,
    };

    let eth = Eth::new(
        ethernet_mac,
        ethernet_dma,
        &mut rx_ring[..],
        &mut tx_ring[..],
        PhyAddress::_1,
        clocks,
        eth_pins,
    )
    .unwrap();
    rprintln!("Ethernet created");
    eth.enable_interrupt();

    return eth;
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
