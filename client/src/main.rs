use embedded_nal::{SocketAddrV4, Ipv4Addr};
use std::{
    fs::File,
    io::{Read, Write},
    str::from_utf8,
};
use tftp_embedded_nal::client::TftpClient;
//smoltcp-nal is supported only for ipv4
use smoltcp_nal::NetworkStack;
use smoltcp::{
    iface::{InterfaceBuilder, NeighborCache},
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

    // create tftp client
    let mut client = TftpClient::new(
        stack,
    );
    /*let mut remote_addr = embedded_nal::SocketAddr::V6(
        SocketAddrV6::new(
            Ipv6Addr::localhost(),
            69, 0, 0));*/
    let mut remote_addr =
        embedded_nal::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::localhost(), 69));

    // read file
    let data = match client.read_file(
        "file2.txt",
        &mut remote_addr)
        {
        Ok(data) => data,
        Err(_) => panic!("can't read file"),
    };
    let mut f = File::create("write_into.txt").unwrap();
    f.write(data.as_slice()).unwrap();
    println!("{:?}", from_utf8(data.as_slice()).unwrap());

    //send file
    //let msg = "Hello, world!".as_bytes();
    let mut msg: Vec<u8> = vec![];
    let mut f = File::open("read_from.txt").unwrap();
    f.read_to_end(&mut msg).unwrap();
    match client.send_file(&mut remote_addr, "file.txt", &msg) {
        Ok(_) => (),
        Err(_) => println!("can't send file"),
    };
}