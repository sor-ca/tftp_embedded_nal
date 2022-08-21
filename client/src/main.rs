use std::net::UdpSocket;
use std::str::from_utf8;
use std::time::Duration;
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8081").expect("couldn't bind to address");
    socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    let buf = b"Hello, world!";
    socket.send_to(&buf[..], "127.0.0.1:8080").expect("couldn't send data");
    let mut r_buf = [0; 516];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut r_buf).expect("didn't receive data");
    let filled_buf = &mut r_buf[..number_of_bytes];
    dbg!(src_addr);
    dbg!(from_utf8(&filled_buf).expect("can't read message"));
}
