use std::{
    fs::File,
    io::Write,
    net::UdpSocket,
    str, thread,
    time::{self, Duration},
};

use message::{ack, data, error};
use tftp::{FileOperation, Message};

fn main() {
    let mut socket = UdpSocket::bind("127.0.0.1:69").expect("couldn't bind to address");
    socket
        .set_read_timeout(Some(Duration::from_secs(100)))
        .unwrap();

    //necessary to add break after several error messages
    loop {
        let mut buf = [0; 516];
        let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("didn't receive data");
        let filled_buf = &mut buf[..number_of_bytes];
        let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
        match message {
            Message::File {
                operation: FileOperation::Write,
                ..
            } => {
                println!("receive wrq message");
                socket = UdpSocket::bind("127.0.0.1:8080").expect("couldn't bind to address");
                socket.connect(src_addr).expect("connect function failed");
                let packet: Vec<u8> = ack(0).into();
                socket.send(packet.as_slice()).expect("couldn't send data");
                break;
            }

            _ => continue,
        }
    }

    let mut f = File::create("write_into.txt").unwrap();
    let mut vec = Vec::with_capacity(1024 * 1024);

    //necessary to add break after several error messages
    loop {
        let mut buf = [0; 516];
        let number_of_bytes = socket.recv(&mut buf).expect("didn't receive data");
        let filled_buf = &mut buf[..number_of_bytes];
        let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Data(block_id, data) => {
                println!("receive data packet");
                dbg!(str::from_utf8(data.as_ref()).expect("can't read message"));
                vec.extend_from_slice(data.as_ref());
                //f.write(data.as_ref()).unwrap();

                let packet: Vec<u8> = ack(block_id).into();
                thread::sleep(time::Duration::from_secs(1));
                socket.send(packet.as_slice()).expect("couldn't send data");
                break;
            }

            _ => continue,
        }
    }
    f.write(vec.as_slice()).unwrap();
}
