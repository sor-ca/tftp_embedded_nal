use std::{
    fs::File,
    io::{Read, Write},
    net::{ToSocketAddrs, UdpSocket},
    thread,
    time::{self, Duration},
};

use std::path::PathBuf;
use ascii::AsciiStr;
use message::{ack, data, error, MyError};
use message::UdpErr::*;
use tftp::{FileOperation, Message, Error};
use embedded_nal::UdpClientStack;
use std_embedded_nal::Stack;

pub struct TftpServer {
    socket: UdpSocket,
}

impl TftpServer
{
    fn new<A: ToSocketAddrs>(socket_addr: A) -> Self {
        Self {
            socket: UdpSocket::bind(socket_addr).expect("couldn't bind server socket to address"),
        }
    }

    fn listen<A: ToSocketAddrs, T: UdpClientStack>(&mut self, socket_addr: A)
        -> Result<[u8; 516], MyError<T>> {
        loop {
            let mut buf = [0; 516];
            let (number_of_bytes, src_addr) = self
                .socket
                .recv_from(&mut buf)
                .map_err(|_| MyError::UdpErr(ReceiveErr))?;

            let filled_buf = &mut buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..])?;
            match message {
                Message::File { operation: _, path, mode: _ } => {
                    println!("receive request");
                    self.socket = UdpSocket::bind(socket_addr).map_err(|_| MyError::UdpErr(BindErr))?;
                    self.socket
                        .connect(src_addr)
                        .map_err(|_| MyError::UdpErr(ConnectErr))?;

                    if !PathBuf::from(path.as_str()).exists() {
                        let packet: Vec<u8> = error(0,
                            AsciiStr::from_ascii(b"invalid access, please check filename").unwrap())
                            .into();
                        self.socket
                            .send(packet.as_slice())
                            .map_err(|_| MyError::UdpErr(SendErr))?;
                            println!("send error message");
                        return Err(MyError::TftpErr(Error::NoPath));
                    }

                    let packet: Vec<u8> = ack(0).into();
                    self.socket
                        .send(packet.as_slice())
                        .map_err(|_| MyError::UdpErr(SendErr))?;

                    let mut out_buf = [0; 516];
                    out_buf.clone_from_slice(&buf);
                    return Ok(out_buf);
                }
                _ => continue,
            }
        }
    }

    fn write<T: UdpClientStack>(&mut self) -> Result<(), MyError<T>> {
        let mut f = File::create("write_into.txt")?;
        let mut vec = Vec::with_capacity(1024 * 1024);

        //necessary to add break after several error messages
        loop {
            let mut buf = [0; 516];
            let number_of_bytes = self.socket.recv(&mut buf)
                .map_err(|_| MyError::UdpErr(ReceiveErr))?;
            let filled_buf = &mut buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..])?;
            match message {
                Message::Data(block_id, data) => {
                    println!("receive data packet");
                    vec.extend_from_slice(data.as_ref());

                    let packet: Vec<u8> = ack(block_id).into();
                    thread::sleep(time::Duration::from_secs(1));
                    self.socket
                        .send(packet.as_slice())
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
        f.write(vec.as_slice()).unwrap();
        Ok(())
    }

    fn read<T: UdpClientStack>(&mut self, filename: &str) -> Result<(), MyError<T>> {
        let mut vec: Vec<u8> = vec![];
        let mut f = File::open(filename)?;
        f.read_to_end(&mut vec)?;
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
                self.socket
                    .send(packet.as_slice())
                    .map_err(|_| MyError::UdpErr(SendErr))?;

                let mut r_buf = [0; 516];
                let number_of_bytes = self.socket.recv(&mut r_buf)
                    .map_err(|_| MyError::UdpErr(ReceiveErr))?;

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

fn main() {
    let mut server = TftpServer::new("127.0.0.1:69");
    server.socket
        .set_read_timeout(Some(Duration::from_secs(100)))
        .unwrap();
    let result = match server.listen::<&str, Stack>("127.0.0.1:8080") {
        Ok(message)  => message,
        Err(_) => panic!("no request"),
    };
    let message: tftp::Message = result[..].try_into().unwrap();
    match message {
        Message::File {
            operation: FileOperation::Write,
            ..
        } => match server.write::<Stack>() {
            Ok(_)  => (),
            Err(_) => panic!("server writing error"),
        },

        Message::File {
            operation: FileOperation::Read,
            path,
            ..
        } => match server.read::<Stack>(path.as_str()) {
            Ok(_)  => (),
            Err(_) => panic!("server reading error"),
        },
        //to satisfy the compiler
        _ => (),
    };
}
