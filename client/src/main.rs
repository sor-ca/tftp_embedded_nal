use std::{
    fs::File,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
    str,
    time::Duration,
};

use ascii::AsciiStr;
use message::{ack, data, error, rrq, wrq};
use tftp::Message;

pub struct TftpClient {
    socket: UdpSocket,
}

impl TftpClient {
    fn new<A: ToSocketAddrs>(socket_addr: A) -> Self {
        Self {
            socket: UdpSocket::bind(socket_addr).expect("couldn't bind to address"),
        }
    }

    fn send_file(&mut self, path: &str, remote: IpAddr) -> io::Result<()> {
        self.socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let packet: Vec<u8> = wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
            .unwrap()
            .into();
        self.socket
            .send_to(packet.as_slice(), SocketAddr::new(remote, 69))
            .expect("couldn't send data");

        //necessary to add break after several error messages
        loop {
            let mut r_buf = [0; 516];
            let (number_of_bytes, src_addr) = self
                .socket
                .recv_from(&mut r_buf)
                .expect("didn't receive data");
            let filled_buf = &mut r_buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
            match message {
                Message::Ack(0) => {
                    println!("receive ack message");
                    self.socket.connect(src_addr).expect("connect failed");
                    break;
                }

                _ => continue,
            }
        }

        let mut vec: Vec<u8> = vec![];
        let mut f = File::open(path).expect("can't open file");
        f.read_to_end(&mut vec).expect("can't read file");
        let mut i = 0;
        let mut j = 512;
        let mut vec_slice: &[u8];
        let mut block_id = 1u16;

        loop {
            vec_slice = if vec.len() > j { &vec[i..j] } else { &vec[i..] };

            let packet: Vec<u8> = data(block_id, vec_slice).unwrap().into();

            loop {
                self.socket
                    .send(packet.as_slice())
                    .expect("couldn't send data");
                let mut r_buf = [0; 516];
                let number_of_bytes = self.socket.recv(&mut r_buf).expect("didn't receive data");
                let filled_buf = &mut r_buf[..number_of_bytes];
                let message =
                    Message::try_from(&filled_buf[..]).expect("can't convert buf to message");

                match message {
                    Message::Ack(id) => {
                        if id == block_id {
                            println!("receive ack message");
                            block_id += 1;
                            break;
                        } else {
                            println!("wrong block id");
                            continue;
                        };
                    }
                    _ => continue,
                }
            }

            if vec.len() <= j {
                println!("file came to end");
                break;
            }
            i += 512;
            j += 512;
        }
        Ok(())
    }

    fn read_file(&mut self, path: &str, remote: IpAddr) -> io::Result<Vec<u8>> {
        self.socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();

        let packet: Vec<u8> = rrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
            .into();
        self.socket
            .send_to(packet.as_slice(), SocketAddr::new(remote, 69))
            .expect("couldn't send data");

        let mut block_id = 1u16;
        let mut vec = Vec::with_capacity(1024 * 1024);
        //i think it is dirty hack but haven't find anymeting else yet
        let mut file_end = false;

        loop {
            let mut r_buf = [0; 516];
            let (number_of_bytes, src_addr) = self
                .socket
                .recv_from(&mut r_buf)
                .expect("didn't receive data");
            let filled_buf = &mut r_buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
            match message {
                Message::Data(id, data) => {
                    if id != block_id {
                        println!("wrong block id");
                        continue;
                    }
                    println!("receive data message");
                    self.socket.connect(src_addr).expect("connect failed");
                    vec.extend_from_slice(data.as_ref());

                    let packet: Vec<u8> = ack(id).into();
                    self.socket
                        .send(packet.as_slice())
                        .expect("couldn't send data");
                    if filled_buf.len() < 516 {
                        println!("file came to end");
                        file_end = true;
                    } else {
                        block_id += 1;
                    };
                    break;
                }
                _ => continue,
            }
        }

        if !file_end {
            //necessary to add break after several error messages
            loop {
                let mut buf = [0; 516];
                let number_of_bytes = self.socket.recv(&mut buf).expect("didn't receive data");
                let filled_buf = &mut buf[..number_of_bytes];
                let message =
                    Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
                //to stop after 3 errors
                let mut error = 0;
                match message {
                    Message::Data(id, data) => {
                        println!("receive data packet");
                        if id != block_id {
                            println!("wrong block id");
                            continue;
                        }
                        vec.extend_from_slice(data.as_ref());

                        let packet: Vec<u8> = ack(block_id).into();
                        //thread::sleep(time::Duration::from_secs(1));
                        self.socket
                            .send(packet.as_slice())
                            .expect("couldn't send data");
                        if number_of_bytes < 516 {
                            //file_end = true;
                            println!("file came to end");
                            break;
                        } else {
                            block_id += 1;
                            continue;
                        }
                    }
                    _ => {
                        error += 1;
                        if error == 3 {
                            println!("3 errors");
                            break;
                        } else {
                            continue;
                        }
                    }
                }
            }
        }
        Ok(vec)
    }
}

fn main() {
    let mut client = TftpClient::new("127.0.0.1:8081");
    let remote = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    //client.send_file("read_from.txt", remote).unwrap();
    let vec = client.read_file("read_from.txt", remote).unwrap();
    let mut f = File::create("write_into.txt").unwrap();
    f.write(vec.as_slice()).unwrap();
}
