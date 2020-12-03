use crate::time;
use coap_lite::{CoapRequest, CoapResponse, Packet};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use once_cell::unsync::{Lazy, OnceCell};
use rtt_target::rprintln;
use smoltcp::iface::{EthernetInterface, EthernetInterfaceBuilder, Neighbor, NeighborCache};
use smoltcp::socket::{
    SocketHandle, SocketSet, SocketSetItem, UdpPacketMetadata, UdpSocket, UdpSocketBuffer,
};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, IpEndpoint, Ipv4Address};
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
static mut NET: OnceCell<
    EthernetInterface<'static, 'static, 'static, &'static mut Eth<'static, 'static>>,
> = OnceCell::new();

static mut SOCKETS_STORAGE: [Option<SocketSetItem>; 2] = [None, None];
static mut SERVER_RX_METADATA_BUFFER: [UdpPacketMetadata; 10] = [UdpPacketMetadata::EMPTY; 10];
static mut SERVER_TX_METADATA_BUFFER: [UdpPacketMetadata; 10] = [UdpPacketMetadata::EMPTY; 10];
static mut SERVER_RX_PAYLOAD_BUFFER: [u8; 2048] = [0; 2048];
static mut SERVER_TX_PAYLOAD_BUFFER: [u8; 2048] = [0; 2048];
static mut SOCKETS: OnceCell<SocketSet<'static, 'static, 'static>> = OnceCell::new();
static mut SERVER_HANDLE: OnceCell<SocketHandle> = OnceCell::new();

pub struct EthPeripherals {
    pub gpioa: hal::gpio::gpioa::Parts,
    pub gpiob: hal::gpio::gpiob::Parts,
    pub gpioc: hal::gpio::gpioc::Parts,
    pub ethernet_mac: hal::stm32::ETHERNET_MAC,
    pub ethernet_dma: hal::stm32::ETHERNET_DMA,
}

pub fn setup_eth(eth_p: EthPeripherals, clocks: hal::rcc::Clocks) {
    rprintln!("Enabling ethernet...");

    let eth_pins = EthPins {
        ref_clk: eth_p.gpioa.pa1,
        md_io: eth_p.gpioa.pa2,
        md_clk: eth_p.gpioc.pc1,
        crs: eth_p.gpioa.pa7,
        tx_en: eth_p.gpiob.pb11,
        tx_d0: eth_p.gpiob.pb12,
        tx_d1: eth_p.gpiob.pb13,
        rx_d0: eth_p.gpioc.pc4,
        rx_d1: eth_p.gpioc.pc5,
    };

    unsafe {
        let eth = Eth::new(
            eth_p.ethernet_mac,
            eth_p.ethernet_dma,
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

pub fn setup_net() {
    const SRC_MAC: [u8; 6] = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    unsafe {
        let neighbor_cache = NeighborCache::new(&mut NEIGHBOUR_STORAGE[..]);
        let ethernet_addr = EthernetAddress(SRC_MAC);

        let eth = ETH.get_mut().expect("ETH not initialized");
        let net = EthernetInterfaceBuilder::new(eth)
            .ethernet_addr(ethernet_addr)
            .ip_addrs(&mut IP_ADDRS[..])
            .neighbor_cache(neighbor_cache)
            .finalize();

        NET.set(net).ok().unwrap();
    }
}

pub fn create_sockets() {
    unsafe {
        let mut sockets = SocketSet::new(&mut SOCKETS_STORAGE[..]);

        let server_socket = UdpSocket::new(
            UdpSocketBuffer::new(
                &mut SERVER_RX_METADATA_BUFFER[..],
                &mut SERVER_RX_PAYLOAD_BUFFER[..],
            ),
            UdpSocketBuffer::new(
                &mut SERVER_TX_METADATA_BUFFER[..],
                &mut SERVER_TX_PAYLOAD_BUFFER[..],
            ),
        );
        let server_handle = sockets.add(server_socket);

        SOCKETS.set(sockets).ok().unwrap();
        SERVER_HANDLE.set(server_handle).ok().unwrap();
    }
}

pub fn handle_request<F>(mut handler: F)
where
    F: FnMut(CoapRequest<IpEndpoint>) -> Option<CoapResponse>,
{
    const PORT: u16 = 5683;

    unsafe {
        let net = NET.get_mut().expect("NET not initialized");
        let sockets = SOCKETS.get_mut().expect("SOCKETS not initialized");
        let server_handle = SERVER_HANDLE.get().expect("SERVER_HANDLE not initialized");
        let time = time::now();

        clear_pending();

        match net.poll(sockets, Instant::from_millis(time as i64)) {
            Ok(true) => {
                let mut socket = sockets.get::<UdpSocket>(*server_handle);

                if !socket.is_open() {
                    socket
                        .bind(PORT)
                        .unwrap_or_else(|e| rprintln!("UDP bind error: {:?}", e));
                }

                let request = match socket.recv() {
                    Ok((data, endpoint)) => {
                        rprintln!("UDP recv from {}", endpoint);

                        let packet = Packet::from_bytes(data).unwrap();
                        let request = CoapRequest::from_packet(packet, endpoint);

                        Some(request)
                    }
                    Err(_) => None,
                };

                if !socket.can_send() {
                    return;
                }

                if let Some(request) = request {
                    let source = request.source.unwrap();

                    match handler(request) {
                        Some(response) => {
                            let packet = response
                                .message
                                .to_bytes()
                                .expect("Cannot convert response to bytes");

                            socket
                                .send_slice(&packet[..], source)
                                .unwrap_or_else(|e| rprintln!("UDP send error: {:?}", e));
                        }
                        None => {
                            rprintln!("No response");
                        }
                    }
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
