use std::net::UdpSocket;
use std::str::from_utf8;
use std::{thread, time};
use std::io::Write;
use std::fs::File;
use message::{Message, FileOperation};
use ascii::AsciiStr;
extern crate alloc;
use alloc::vec::Vec;
use std::time::Duration;
//use std::net::SocketAddr;

fn main() {
    let mut socket = UdpSocket::bind("127.0.0.1:69").expect("couldn't bind to address");
    socket.set_read_timeout(Some(Duration::from_secs(100))).unwrap();
    let mut buf = [0; 516];
    //necessary to add break after several error messages
    loop {
        let (_number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("didn't receive data");
        let message = Message::try_from(&buf[..]).expect("can't convert buf to message");
        match message {
            Message::File {operation: FileOperation::Write, ..} => {
                println!("receive wrq message");
                socket = UdpSocket::bind("127.0.0.1:8080").expect("couldn't bind to address");
                socket.connect(src_addr).expect("connect function failed");
                let packet: Vec<u8> = Message::ack(0).into();
                socket.send(packet.as_slice()).expect("couldn't send data");
                break;
            },

            _ => continue,
        }
    }
    

    let mut f = File::create("write_into.txt").unwrap();

    //necessary to add break after several error messages
    loop {
        socket.recv(&mut buf).expect("didn't receive data");
        let message = Message::try_from(&buf[..]).expect("can't convert buf to message");
        match message {
            Message::Data(block_id, data) => {
                println!("receive data packet");
                dbg!(from_utf8(data.as_ref()).expect("can't read message"));

                let packet: Vec<u8> = Message::ack(block_id).into();                
                f.write(data.as_ref()).unwrap();
                thread::sleep(time::Duration::from_secs(1));
                socket.send(packet.as_slice()).expect("couldn't send data");
                break;
            },

            _ => continue,
        }
    }    
}

