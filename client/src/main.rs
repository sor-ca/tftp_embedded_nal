use ascii::AsciiStr;
use message::Message;
use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;
use std::str::from_utf8;
extern crate alloc;
use alloc::vec::Vec;
use std::time::Duration;
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8081").expect("couldn't bind to address");
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let path = "read_from.txt";

    let packet: Vec<u8> = Message::wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
        .unwrap()
        .into();
    socket
        .send_to(packet.as_slice(), "127.0.0.1:69")
        .expect("couldn't send data");

    let mut r_buf = [0; 516];
    //necessary to add break after several error messages
    loop {
        let (_number_of_bytes, src_addr) =
            socket.recv_from(&mut r_buf).expect("didn't receive data");
        let message = Message::try_from(&r_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(0) => {
                println!("receive ack message");
                socket.connect(src_addr).expect("connect function failed");
                break;
            }

            _ => continue,
        }
    }

    let mut buf = [0; 512];
    let mut f = File::open(path).unwrap();
    f.read(&mut buf).unwrap();
    let mut block_id = 1u16;
    let packet: Vec<u8> = Message::data(block_id, &buf[..]).unwrap().into();

    loop {
        socket.send(packet.as_slice()).expect("couldn't send data");
        socket.recv(&mut r_buf).expect("didn't receive data");
        let message = Message::try_from(&r_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(id) => {
                if id == block_id {
                    println!("receive ack message");
                    break;
                } else {
                    println!("wrong block id");
                    continue;
                };
            }
            _ => continue,
        }
    }
}
