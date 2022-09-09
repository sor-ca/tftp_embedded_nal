/// that modules represents your library
mod embedded_tftp {
    use embedded_nal::{SocketAddr, UdpClientStack, UdpFullStack};
        //SocketAddrV6, Ipv6Addr};
    use ascii::AsciiStr;
    use message::{ack, data, rrq, wrq};
    use message::UdpErr::*;
    use tftp::{Message};
    use message::MyError;
    use nb;

    pub struct TftpClient<T>
    where
        T: UdpClientStack + UdpFullStack,
    {
        udp: T,
        pub socket: T::UdpSocket,
    }

    impl<T> TftpClient<T>
    where
        T: UdpClientStack + UdpFullStack,
    {
        pub fn new(mut udp: T) -> Self {
            let mut socket = udp.socket().unwrap();
            udp.bind(&mut socket, 8081).unwrap();

            Self {
                udp: udp,
                socket: socket,
            }
        }

        pub fn read_file(&mut self, path: &str, remote_addr: &mut SocketAddr) -> Result<Vec<u8>, MyError<T>>
        {
            let packet: Vec<u8> = rrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
            .into();
            println!("create packet");
            self.udp
                .send_to(&mut self.socket, *remote_addr, packet.as_slice())
                .unwrap();
                //.map_err(|e: nb::Error<<T>::Error>| MyError::UdpClientStackErrnb(e))?;
                //.map_err(|_| MyError::UdpErr(SendErr))?;
            println!("send request");

            let mut block_id = 1u16;
            let mut vec = Vec::with_capacity(1024 * 1024);
            let mut file_end = false;

            loop {
                let mut r_buf = [0; 516];
                let result = self.udp
                    .receive(&mut self.socket, &mut r_buf);
                let (number_of_bytes, src_addr) = match result {
                    Ok(n,) => n,
                    Err(nb::Error::WouldBlock) => continue,
                    Err(_) => panic!("no request"),
                };
                println!("receive message");

                let filled_buf = &mut r_buf[..number_of_bytes];
                let message = Message::try_from(&filled_buf[..])?;
                match message {
                    Message::Data(id, data) => {
                        if id != block_id {
                            println!("wrong block id");
                            continue;
                        }
                        println!("receive data message");
                        *remote_addr = src_addr;
                        vec.extend_from_slice(data.as_ref());

                        let packet: Vec<u8> = ack(id).into();
                        self.udp.send_to(&mut self.socket, *remote_addr, packet.as_slice())
                            //.map_err(|e: nb::Error<<T>::Error>| MyError::UdpClientStackErrnb(e))?;
                            .map_err(|_| MyError::UdpErr(SendErr))?;

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
                    let mut r_buf = [0; 516];
                    let result = self.udp
                        .receive(&mut self.socket, &mut r_buf);
                    let (number_of_bytes, _src_addr) = match result {
                        Ok(n,) => n,
                        Err(nb::Error::WouldBlock) => continue,
                        Err(_) => panic!("no request"),
                    };

                    let filled_buf = &mut r_buf[..number_of_bytes];
                    let message = Message::try_from(&filled_buf[..])?;

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
                            self.udp
                                .send_to(&mut self.socket, *remote_addr, packet.as_slice())
                                //.map_err(|e: nb::Error<<T>::Error>| MyError::UdpClientStackErrnb(e))?;
                                .map_err(|_| MyError::UdpErr(SendErr))?;

                            if number_of_bytes < 516 {
                                //file_end = true;
                                println!("file came to end");
                                break;
                            } else {
                                block_id += 1;
                                error = 0;
                                continue;
                            }
                        }
                        _ => {
                            if error == 3 {
                                println!("3 errors");
                                break;
                            } else {
                                error += 1;
                                continue;
                            }
                        }
                    }
                }
            }
            Ok(vec)
        }

        pub fn send_file(&mut self, remote_addr: &mut SocketAddr, path: &str, data_vec: &[u8]) -> Result<(), MyError<T>>
        {
            let mut packet: Vec<u8> = wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true).into();
            self.udp
                .send_to(&mut self.socket, *remote_addr, packet.as_slice())
                .map_err(|e: nb::Error<<T>::Error>| MyError::UdpClientStackErrnb(e))?;
                //.map_err(|_| MyError::UdpErr(SendErr))?;

                let mut i = 0;
                let mut j = 512;
                let mut vec_slice: &[u8];
                let mut block_id = 0u16;

                loop {
                    vec_slice = if data_vec.len() > j { &data_vec[i..j] } else { &data_vec[i..] };

                    loop {
                        let mut r_buf = [0; 516];
                        let result = self.udp
                            .receive(&mut self.socket, &mut r_buf);
                        let (number_of_bytes, src_addr) = match result {
                            Ok(n,) => n,
                            Err(nb::Error::WouldBlock) => continue,
                            Err(_) => panic!("no request"),
                        };

                        let filled_buf = &mut r_buf[..number_of_bytes];
                        let message = Message::try_from(&filled_buf[..])?;
                        match message {
                            Message::Ack(id) => {
                                    if id == block_id {
                                        println!("receive ack message");
                                        block_id += 1;
                                        packet = data(block_id, vec_slice)?.into();
                                        self.udp
                                            .send_to(&mut self.socket, src_addr, packet.as_slice())
                                            .map_err(|e: nb::Error<<T>::Error>| MyError::UdpClientStackErrnb(e))?;
                                            //.map_err(|_| MyError::UdpErr(SendErr))?;
                                        break;
                                    } else {
                                        println!("wrong block id");
                                        continue;
                                    };
                                }
                            _ => continue,
                        }
                    }if data_vec.len() <= j {
                        println!("file came to end");
                        break;
                    }
                    i += 512;
                    j += 512;
                }
                Ok(())
            }
        }
    }


// following is a user who uses your library
use embedded_nal::{SocketAddrV6, Ipv6Addr};
    //SocketAddrV4, IpAddr, Ipv4Addr, SocketAddr};
use embedded_tftp::TftpClient;
use std_embedded_nal::{Stack};
use std::{
    fs::File,
    io::{Read, Write},
};

fn main() {
    // create concrete implementation
    let std_stack = Stack::default();

    // create tftp client
    let mut client = TftpClient::new(
        std_stack,
    );
    let mut remote_addr = embedded_nal::SocketAddr::V6(
        SocketAddrV6::new(
            Ipv6Addr::localhost(),
            69, 0, 0));

    // read file
    /*let data = match client.read_file(
        "file2.txt",
        //&mut SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::localhost(), 69)))
        &mut remote_addr)
        {
        Ok(data) => data,
        Err(_) => panic!("can't read file"),
    };
    println!("{:?}", data);*/

    //send file
    let mut msg: Vec<u8> = vec![];
    let mut f = File::open("read_from.txt").unwrap();
    f.read_to_end(&mut msg).unwrap();
    //let msg = "Hello, world!".as_bytes();
    match client.send_file(&mut remote_addr, "file.txt", &msg) {
        Ok(_) => (),
        Err(_) => println!("can't send file"),
    };
}