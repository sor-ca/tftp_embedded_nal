use std::{
    fs::File,
    io::{Read, Write},
};
//use std_embedded_nal::Stack;
use smoltcp_nal::{UdpSocket, NetworkError, NetworkStack};
use tftp_embedded_nal::server::{TftpServer, RequestType};
use heapless::Vec;
use smoltcp::iface::{InterfaceBuilder, NeighborCache, Interface};
use smoltcp::socket::udp::{Socket, PacketMetadata, PacketBuffer};
use smoltcp::phy::{Loopback, Medium};
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv6Address, IpAddress};
use embedded_time::{clock::{Clock, Error}, fraction::Fraction, Instant};

struct SomeClock;

impl Clock for SomeClock {
    type T = u32;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 16_000_000);
    fn try_now(&self) -> Result<Instant<Self>, Error> {
        Ok(Instant::new(100))
    }
}

fn main() {
    //let std_stack = Stack::default();
    let device = Loopback::new(Medium::Ethernet);
    let hw_addr = EthernetAddress::default();
    let mut neighbor_cache_storage = [None; 8];
    let mut neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);
    let ip_addrs = [
        IpCidr::new(IpAddress::Ipv6(Ipv6Address::LOOPBACK), 64),
    ];

    //let iface = InterfaceBuilder::new(device, vec![])
    let iface = InterfaceBuilder::new()
        .hardware_addr(hw_addr.into())
        .neighbor_cache(neighbor_cache)
        .ip_addrs(ip_addrs)
        .finalize(&mut device);

    let udp_rx_buffer = PacketBuffer::new(
        vec![PacketMetadata::EMPTY, PacketMetadata::EMPTY],
        vec![0; 65535],
    );
    let udp_tx_buffer = PacketBuffer::new(
        vec![PacketMetadata::EMPTY, PacketMetadata::EMPTY],
        vec![0; 65535],
    );
    let udp_socket = Socket::new(udp_rx_buffer, udp_tx_buffer);
    iface.add_socket(udp_socket);

    let clock: SomeClock;

    let stack = NetworkStack::new(iface, clock);

    let mut server = TftpServer::new(
        stack,
    );
    println!("create server");
    let (req_type, src_addr, filename) = match server.listen() {
        Ok(result)  => result,
        Err(_) => panic!("no request"),
    };

    match req_type {
        RequestType::Write => match server.write(src_addr) {
            Ok(vec)  => {
                let mut f = File::create(filename.as_str()).unwrap();
                f.write(vec.as_slice()).unwrap();
            },
            Err(_) => println!("server writing error"),
        },
        RequestType::Read => {
            let mut vec: std::vec::Vec<u8> = vec![];
            let mut f = File::open(filename.as_str()).unwrap();
            f.read_to_end(&mut vec).unwrap();
            let mut h_vec: Vec<u8, {10 * 1024}> = heapless::Vec::from_slice(&vec[..]).unwrap();

            match server.read(src_addr, &mut h_vec) {
                Ok(_)  => (),
                Err(_) => println!("server reading error"),
            }
        },
    }
}