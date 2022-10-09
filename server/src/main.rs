use std::{
    fs::File,
    io::{Read, Write},
};
use tftp_embedded_nal::server::{TftpServer, RequestType};
use heapless::Vec;

//smoltcp-nal is supported only for ipv4
use smoltcp_nal::{UdpSocket as Socket, NetworkError, NetworkStack};
use smoltcp::{
    iface::{InterfaceBuilder, NeighborCache, Interface},
    socket::{UdpSocket, UdpSocketBuffer},
    storage::PacketMetadata,
    phy::{Loopback, Medium},
    wire::{EthernetAddress, IpCidr, Ipv4Address, IpAddress},
};
use embedded_time::{
    clock::{Clock, Error},
    fraction::Fraction,
    Instant
};

struct SysClock;

impl Clock for SysClock {
    type T = u32;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 16_000_000);
    fn try_now(&self) -> Result<Instant<Self>, Error> {
        Ok(Instant::new(100))
    }
}

fn main() {
    let device = Loopback::new(Medium::Ethernet);
    let hw_addr = EthernetAddress::default();
    let mut neighbor_cache_storage = [None; 8];
    let mut neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);
    /*let ip_addrs = [
        IpCidr::new(IpAddress::Ipv6(Ipv6Address::LOOPBACK), 64),
    ];*/
    let ip_addrs = [
        IpCidr::new(IpAddress::Ipv4(Ipv4Address::new(127,0, 0, 1)), 32),
    ];

    let mut iface = InterfaceBuilder::new(device, vec![])
        .hardware_addr(hw_addr.into())
        .neighbor_cache(neighbor_cache)
        .ip_addrs(ip_addrs)
        .finalize();
    println!("interface");

    let udp_rx_buffer = UdpSocketBuffer::new(
        vec![PacketMetadata::EMPTY, PacketMetadata::EMPTY],
        vec![0; 516],
    );
    let udp_tx_buffer = UdpSocketBuffer::new(
        vec![PacketMetadata::EMPTY, PacketMetadata::EMPTY],
        vec![0; 516],
    );
    let udp_socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);
    println!("create socket");

    iface.add_socket(udp_socket);
    println!("add socket");

    let clock = SysClock;
    println!("clock");

    let stack = NetworkStack::new(iface, clock);
    println!("stack");

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