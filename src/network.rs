use crate::time;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::Mutex;
use once_cell::unsync::{Lazy, OnceCell};
use rtt_target::rprintln;
use smoltcp::iface::{EthernetInterface, EthernetInterfaceBuilder, Neighbor, NeighborCache};
use smoltcp::socket::{SocketHandle, SocketSet, SocketSetItem, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
use smoltcp::Error;
use stm32_eth::{Eth, EthPins, PhyAddress, RingEntry, RxDescriptor, TxDescriptor};
use stm32f4xx_hal as hal;

static ETH_PENDING: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

static mut RX_RING: Lazy<[RingEntry<RxDescriptor>; 8]> = Lazy::new(Default::default);
static mut TX_RING: Lazy<[RingEntry<TxDescriptor>; 2]> = Lazy::new(Default::default);
static mut ETH: OnceCell<Eth<'static, 'static>> = OnceCell::new();

static mut IP_ADDRS: Lazy<[IpCidr; 1]> = Lazy::new(|| {
    let local_addr = Ipv4Address::new(192, 168, 1, 100);
    let ip_addr = IpCidr::new(IpAddress::from(local_addr), 24);

    [ip_addr]
});
static mut NEIGHBOUR_STORAGE: [Option<(IpAddress, Neighbor)>; 16] = [None; 16];
static mut IFACE: OnceCell<
    EthernetInterface<'static, 'static, 'static, &'static mut Eth<'static, 'static>>,
> = OnceCell::new();

static mut SOCKETS_STORAGE: [Option<SocketSetItem>; 2] = [None, None];
static mut SERVER_RX_BUFFER: [u8; 2048] = [0; 2048];
static mut SERVER_TX_BUFFER: [u8; 2048] = [0; 2048];
static mut SOCKETS: OnceCell<SocketSet<'static, 'static, 'static>> = OnceCell::new();
static mut SERVER_HANDLE: OnceCell<SocketHandle> = OnceCell::new();

pub fn setup_eth(
    gpioa: hal::gpio::gpioa::Parts,
    gpiob: hal::gpio::gpiob::Parts,
    gpioc: hal::gpio::gpioc::Parts,
    clocks: hal::rcc::Clocks,
    ethernet_mac: hal::stm32::ETHERNET_MAC,
    ethernet_dma: hal::stm32::ETHERNET_DMA,
) {
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

    unsafe {
        let eth = Eth::new(
            ethernet_mac,
            ethernet_dma,
            &mut RX_RING[..],
            &mut TX_RING[..],
            PhyAddress::_1,
            clocks,
            eth_pins,
        )
        .unwrap();
        eth.enable_interrupt();

        ETH.set(eth).ok().unwrap();
        rprintln!("Ethernet created");
    }
}

pub fn setup_iface() {
    const SRC_MAC: [u8; 6] = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    unsafe {
        let neighbor_cache = NeighborCache::new(&mut NEIGHBOUR_STORAGE[..]);
        let ethernet_addr = EthernetAddress(SRC_MAC);

        let eth = ETH.get_mut().expect("ETH not initialized");
        let iface = EthernetInterfaceBuilder::new(eth)
            .ethernet_addr(ethernet_addr)
            .ip_addrs(&mut IP_ADDRS[..])
            .neighbor_cache(neighbor_cache)
            .finalize();

        IFACE.set(iface).ok().unwrap();
    }
}

pub fn create_sockets() {
    unsafe {
        let mut sockets = SocketSet::new(&mut SOCKETS_STORAGE[..]);

        let server_socket = TcpSocket::new(
            TcpSocketBuffer::new(&mut SERVER_RX_BUFFER[..]),
            TcpSocketBuffer::new(&mut SERVER_TX_BUFFER[..]),
        );
        let server_handle = sockets.add(server_socket);

        SOCKETS.set(sockets).ok().unwrap();
        SERVER_HANDLE.set(server_handle).ok().unwrap();
    }
}

pub fn handle_request<'a, F>(f: F)
where
    F: FnOnce() -> &'a str,
{
    const PORT: u16 = 3000;

    unsafe {
        let iface = IFACE.get_mut().expect("IFACE not initialized");
        let sockets = SOCKETS.get_mut().expect("SOCKETS not initialized");
        let server_handle = SERVER_HANDLE.get().expect("SERVER_HANDLE not initialized");
        let time = time::now();

        clear_pending();

        match iface.poll(sockets, Instant::from_millis(time as i64)) {
            Ok(true) => {
                let mut socket = sockets.get::<TcpSocket>(*server_handle);

                if !socket.is_open() {
                    socket
                        .listen(PORT)
                        .unwrap_or_else(|e| rprintln!("TCP listen error: {:?}", e));
                }

                if socket.can_send() {
                    writeln!(socket, "{}", f())
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
                if e != Error::Unrecognized {
                    rprintln!("Error: {:?}", e);
                }
            }
        }
    }
}

pub fn set_pending() {
    cortex_m::interrupt::free(|cs| {
        let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
        *eth_pending = true;
    });
}

pub fn clear_pending() {
    cortex_m::interrupt::free(|cs| {
        let mut eth_pending = ETH_PENDING.borrow(cs).borrow_mut();
        *eth_pending = false;
    });
}
