/*use std::{
    fs::File,
    io::{Read, Write},
    //str::from_utf8,
    thread,
    //time::{self, Duration},
};*/

use std::path::PathBuf;
use ascii::{AsciiStr, AsciiString};
use nb;
use message::{ack, data, error, MyError};
use message::UdpErr::*;
use tftp::{FileOperation, Message, Error};
use embedded_nal::{UdpClientStack, UdpFullStack, SocketAddr};
    //IpAddr, Ipv4Addr, Ipv6Addr};
//use std_embedded_nal::Stack;

pub struct TftpServer<T>
where T: UdpClientStack + UdpFullStack,
{
    udp: T,
    pub socket: T::UdpSocket,
}

pub enum RequestType {
    Read,
    Write,
}


impl<T> TftpServer<T>
where T: UdpClientStack + UdpFullStack,
{
    pub fn new(mut udp: T) -> Self {
        let mut socket = udp.socket().unwrap();
        udp.bind(&mut socket, 69).unwrap();

        Self {
            udp: udp,
            socket: socket,
        }
    }

    pub fn new_connected(mut udp: T, remote: &SocketAddr) -> Self {
        let mut socket = udp.socket().unwrap();
        udp.connect(&mut socket, *remote).unwrap();

        Self {
            udp: udp,
            socket: socket,
        }
    }

    pub fn listen(&mut self)
        //-> Result<(SocketAddr, [u8; 516]), MyError<T>> {
            -> Result<(RequestType, SocketAddr, AsciiString), MyError<T>> {
        loop {
            let mut buf = [0; 516];
            let result = self.udp
                .receive(&mut self.socket, &mut buf);
            let (number_of_bytes, src_addr) = match result {
                Ok(n,) => n,
                Err(nb::Error::WouldBlock) => continue,
                Err(_) => panic!("no request"),
            };

            println!("scr_addr {:?}", src_addr);
            let mut filename = AsciiString::new();

            let filled_buf = &mut buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..])?;
            match message {
                Message::File { operation, path, mode: _ } => {
                    println!("receive request");
                    if !PathBuf::from(path.as_str()).exists() {
                        println!("no path");
                        let packet: Vec<u8> = error(0,
                            AsciiStr::from_ascii(b"invalid access, please check filename").unwrap())
                            .into();
                        self.udp
                            .send_to(&mut self.socket, src_addr, packet.as_slice())
                            .map_err(|_| MyError::UdpErr(SendErr))?;
                            println!("send error message");
                        return Err(MyError::TftpErr(Error::NoPath));
                    } else {
                        filename = path.to_ascii_string();
                    }
                    match operation {
                        FileOperation::Read => return Ok((RequestType::Read, src_addr, filename)),
                        FileOperation::Write => return Ok((RequestType::Write, src_addr, filename)),
                    };

                    //let mut out_buf = [0; 516];
                    //out_buf.clone_from_slice(&buf);
                    //return Ok((src_addr, filename));
                }
                _ => continue,
            }
        }
    }

    pub fn write(&mut self, remote: SocketAddr) -> Result<Vec<u8>, MyError<T>> {
        self.udp
            .connect(&mut self.socket, remote)
            .unwrap();
            //.map_err(|_| MyError::UdpErr(ConnectErr))?;
        println!("connect socket");
        let mut vec = Vec::with_capacity(1024 * 1024);
        let packet: Vec<u8> = ack(0).into();
        self.udp
            .send(&mut self.socket, packet.as_slice())
            .unwrap();
            //.map_err(|_| MyError::UdpErr(SendErr))?;
        println!("send ack");

        //necessary to add break after several error messages
        loop {
            let mut buf = [0; 516];
            let result = self.udp
                .receive(&mut self.socket, &mut buf);
            let (number_of_bytes, _src_addr) = match result {
                Ok(n) => n,
                Err(nb::Error::WouldBlock) => continue,
                Err(_) => panic!("no request"),
            };
            let filled_buf = &mut buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..])?;

            match message {
                Message::Data(block_id, data) => {
                    println!("receive data packet");
                    vec.extend_from_slice(data.as_ref());

                    let packet: Vec<u8> = ack(block_id).into();
                    //thread::sleep(time::Duration::from_secs(1));
                    self.udp
                        .send(&mut self.socket, packet.as_slice())
                        .map_err(|_| MyError::UdpErr(SendErr))?;

                    if number_of_bytes < 516 {
                        break;
                    } else {
                        continue;
                    }
                }

                _ => continue,
            }
        }
        Ok(vec)
    }

    pub fn read(&mut self, remote: SocketAddr, vec: &mut Vec<u8>) -> Result<(), MyError<T>> {
        self.udp
            .connect(&mut self.socket, remote)
            .unwrap();
            //.map_err(|_| MyError::UdpErr(ConnectErr))?;
        println!("connect socket");

        let mut i = 0;
        let mut j = 512;
        let mut vec_slice: &[u8];
        let mut block_id = 1u16;

        loop {
            vec_slice = if vec.len() > j { &vec[i..j] } else { &vec[i..] };
            let packet: Vec<u8> = match data::<T>(block_id, vec_slice) {
                Ok(buf) => buf.into(),
                Err(_) => panic!(),
            };

            loop {
                self.udp
                    .send(&mut self.socket, packet.as_slice())
                    .unwrap();
                    //.map_err(|_| MyError::UdpErr(SendErr))?;

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
}