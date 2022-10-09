/// that modules represents your library
//mod embedded_tftp {
    use embedded_nal::{SocketAddr, UdpClientStack, UdpFullStack};
    use ascii::AsciiStr;
    use crate::message::{ack, data, rrq, wrq, to_heapless,
        UdpErr::*,
        MyError};
    use tftp::{Message};
    use heapless::Vec;
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

        pub fn read_file(&mut self, path: &str, remote_addr: &mut SocketAddr) -> Result<Vec<u8, {10 * 1024}>, MyError<T>>
        {
            let mut packet: Vec<u8, 516> = to_heapless(
                rrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true));
            //.into();
            println!("create packet");
            self.udp
                .send_to(&mut self.socket, *remote_addr, packet.as_slice())
                //.unwrap();
                .map_err(|e| MyError::UdpErr(SendErr(e)))?;
            println!("send request");

            let mut block_id = 1u16;

            let mut vec: Vec<u8, {10 * 1024}> = Vec::new();
            //let mut file_end = false;
            let mut error = 0;

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
                        vec.extend_from_slice(data.as_ref()).unwrap();

                        packet = to_heapless(ack(id));
                        self.udp.send_to(&mut self.socket, *remote_addr, packet.as_slice())
                            .map_err(|e| MyError::UdpErr(SendErr(e)))?;

                        if filled_buf.len() < 516 {
                            println!("file came to end");
                            break;
                            //file_end = true;
                        } else {
                            block_id += 1;
                            error = 0;
                            continue;
                        };
                    }
                    _ => {
                        if error == 2 {
                            println!("2 errors");
                            break;
                        } else {
                            error += 1;
                            self.udp.send_to(&mut self.socket, *remote_addr, packet.as_slice())
                                .map_err(|e| MyError::UdpErr(SendErr(e)))?;
                            continue;
                        }
                    }
                }
            }

            /*if !file_end {
                //necessary to add break after several error messages
                loop {
                    let mut r_buf = [0; 516];
                    let result = self.udp
                        .receive(&mut self.socket, &mut r_buf);
                    let (number_of_bytes, src_addr) = match result {
                        Ok(n,) => n,
                        Err(nb::Error::WouldBlock) => continue,
                        Err(_) => panic!("no request"),
                    };
                    if src_addr != *remote_addr {
                        continue;
                    }

                    let filled_buf = &mut r_buf[..number_of_bytes];
                    let message = Message::try_from(&filled_buf[..])?;

                    match message {
                        Message::Data(id, data) => {
                            println!("receive data packet");
                            if id != block_id {
                                println!("wrong block id");
                                continue;
                            }
                            vec.extend_from_slice(data.as_ref()).unwrap();

                            packet = to_heapless(ack(block_id));
                            //.into();
                            self.udp
                                .send_to(&mut self.socket, *remote_addr, packet.as_slice())
                                .map_err(|e| MyError::UdpErr(SendErr(e)))?;

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
                            if error == 2 {
                                println!("3 errors");
                                break;
                            } else {
                                error += 1;
                                self.udp.send_to(&mut self.socket, *remote_addr, packet.as_slice())
                                    .map_err(|e| MyError::UdpErr(SendErr(e)))?;
                                continue;
                            }
                        }
                    }
                }
            }*/
            Ok(vec)
        }

        pub fn send_file(&mut self, remote_addr: &mut SocketAddr, path: &str, data_vec: &[u8]) -> Result<(), MyError<T>>
        {
            let mut packet: Vec<u8, 516> = to_heapless(
                wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true));
                //.into();
            self.udp
                .send_to(&mut self.socket, *remote_addr, packet.as_slice())
                .map_err(|e| MyError::UdpErr(SendErr(e)))?;

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
                                        packet = to_heapless(
                                            data(block_id, vec_slice)?);
                                        //.into();
                                        self.udp
                                            .send_to(&mut self.socket, src_addr, packet.as_slice())
                                            .map_err(|e| MyError::UdpErr(SendErr(e)))?;
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
    //}
