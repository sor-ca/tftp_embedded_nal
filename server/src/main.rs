use std::net::UdpSocket;
use std::str::from_utf8;
use std::{thread, time};
//use std::net::SocketAddr;

fn main() {
    let start_socket = UdpSocket::bind("127.0.0.1:69").expect("couldn't bind to address");
    let mut buf = [0; 516];
    let (number_of_bytes, src_addr) = start_socket.recv_from(&mut buf).expect("didn't receive data");
    dbg!(src_addr);
    let mut filled_buf = &mut buf[..number_of_bytes];
    dbg!(from_utf8(&filled_buf).expect("can't read message"));
    let new_socket = UdpSocket::bind("127.0.0.1:8080").expect("couldn't bind to address");
    new_socket.send_to(&filled_buf, src_addr).expect("couldn't send data");

    loop {
        //let mut buf = [0; 516];
        //let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("didn't receive data");
        let (number_of_bytes, src_addr) = new_socket.recv_from(&mut buf).expect("didn't receive data");
        dbg!(src_addr);
        filled_buf = &mut buf[..number_of_bytes];
        dbg!(from_utf8(&filled_buf).expect("can't read message"));
        thread::sleep(time::Duration::from_secs(1));
        //let message_text = from_utf8(&buf[4..]).expect("can't read message");
        //println!("server receives {}", message_text);
        new_socket.send_to(&filled_buf, src_addr).expect("couldn't send data");
    }    
}

