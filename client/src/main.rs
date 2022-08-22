use std::net::UdpSocket;
use std::str::from_utf8;
use std::io::Read;
use std::fs::File;
use message::Message;
use ascii::AsciiStr;
extern crate alloc;
use alloc::vec::Vec;
//use std::time::Duration;
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8081").expect("couldn't bind to address");
    //socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();

    let path = "read_from.txt";

    let packet: Vec<u8> = Message::wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true).unwrap().into();
    
    socket.send_to(packet.as_slice(), "127.0.0.1:69").expect("couldn't send data");
    let mut r_buf = [0; 516];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut r_buf).expect("didn't receive data");
    let mut filled_buf = &mut r_buf[..number_of_bytes];
    dbg!(src_addr);
    dbg!(from_utf8(&filled_buf).expect("can't read message"));

    let mut buf = [0; 516];
    let mut f = File::open(path).unwrap();
    f.read(&mut buf).unwrap();
    
    socket.send_to(&buf[..], src_addr).expect("couldn't send data");
    let (number_of_bytes, src_addr) = socket.recv_from(&mut r_buf).expect("didn't receive data");
    filled_buf = &mut r_buf[..number_of_bytes];
    dbg!(src_addr);
    dbg!(from_utf8(&filled_buf).expect("can't read message"));
}
