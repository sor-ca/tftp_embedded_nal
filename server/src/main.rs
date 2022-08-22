use std::net::UdpSocket;
use std::str::from_utf8;
use std::{thread, time};
use std::io::Write;
use std::fs::File;
use message::{Message, FileOperation};
//use std::net::SocketAddr;

fn main() {
    let start_socket = UdpSocket::bind("127.0.0.1:69").expect("couldn't bind to address");
    //start_socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut buf = [0; 516];
    let (number_of_bytes, src_addr) = start_socket.recv_from(&mut buf).expect("didn't receive data");
    dbg!(src_addr);
    dbg!(&buf);
    let message = Message::try_from(&buf[..]).expect("can't convert buf to message");
    match message {
        Message::File {operation: FileOperation::Write, ..} => println!("receive wrq message"),
        _ => println!("receive wrong message"),
    }
    //let mut filled_buf = &mut buf[..number_of_bytes];
    let mut f = File::create("write_into.txt").unwrap();
    //f.write(&mut filled_buf).unwrap();
    //dbg!(from_utf8(&filled_buf).expect("can't read message"));

    //i don't like to create new socket, but i don't understand how to change port
    let new_socket = UdpSocket::bind("127.0.0.1:8080").expect("couldn't bind to address");
    //new_socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    new_socket.send_to(&mut buf, src_addr).expect("couldn't send data");

    loop {
        let (number_of_bytes, src_addr) = new_socket.recv_from(&mut buf).expect("didn't receive data");
        dbg!(src_addr);
        let mut filled_buf = &mut buf[..number_of_bytes];
        dbg!(from_utf8(&filled_buf).expect("can't read message"));

        f.write(&mut filled_buf).unwrap();
        thread::sleep(time::Duration::from_secs(1));
        new_socket.send_to(&filled_buf, src_addr).expect("couldn't send data");
    }    
}

