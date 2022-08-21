use std::net::UdpSocket;
use std::str::from_utf8;
use std::io::Read;
use std::fs::File;
//use tftp::Message;
//use std::time::Duration;
fn main() {
    let mut buf = [0; 516];

    let socket = UdpSocket::bind("127.0.0.1:8081").expect("couldn't bind to address");
    //socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

    let mut f = File::open("read_from.txt").unwrap();
    f.read(&mut buf).unwrap();
    
    socket.send_to(&buf[..], "127.0.0.1:69").expect("couldn't send data");
    let mut r_buf = [0; 516];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut r_buf).expect("didn't receive data");
    let mut filled_buf = &mut r_buf[..number_of_bytes];
    dbg!(src_addr);
    dbg!(from_utf8(&filled_buf).expect("can't read message"));

    let mut text = "second_message".as_bytes();
    text.read(&mut buf).expect("can't read");
    
    socket.send_to(&buf[..], src_addr).expect("couldn't send data");
    let (number_of_bytes, src_addr) = socket.recv_from(&mut r_buf).expect("didn't receive data");
    filled_buf = &mut r_buf[..number_of_bytes];
    dbg!(src_addr);
    dbg!(from_utf8(&filled_buf).expect("can't read message"));
}
