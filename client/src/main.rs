use nb;
use std::io;
use std::{
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

use ascii::AsciiStr;
use message::{ack, data, error, rrq, wrq};
use tftp::Message;

pub struct TftpClient {
    local_ip: IpAddr,
}

impl TftpClient {
    pub fn new(local_ip: IpAddr) -> Self {
        Self { local_ip }
    }
    //I don't think that to copy file into vec is a good idea.
    //Usually in such cases, buffers are used
    pub fn read_file(&mut self, filename: &str) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        let mut f = File::open(filename).expect("can't open file");
        f.read_to_end(&mut buf).expect("can't read file");
        buf
    }

    fn socket(&mut self) -> io::Result<UdpSocket> {
        Ok(UdpSocket::bind(SocketAddr::new(self.local_ip, 0)).expect("couldn't bind to address"))
    }

    fn connect(&mut self, socket: &mut UdpSocket, remote: SocketAddr) -> io::Result<()> {
        socket.connect(remote).expect("connect function failed");
        Ok(())
    }

    fn send(
        &mut self,
        socket: &mut UdpSocket,
        buffer: &[u8],
    ) -> nb::Result<(), nb::Error<io::Error>> {
        socket.send(buffer).expect("couldn't send data");
        Ok(())
    }

    fn receive(
        &mut self,
        socket: &mut UdpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(usize, SocketAddr), nb::Error<io::Error>> {
        let (number_of_bytes, src_addr) = socket.recv_from(buffer).expect("Didn't receive data");
        Ok((number_of_bytes, src_addr))
    }

    fn close(&mut self, socket: UdpSocket) -> io::Result<()> {
        todo!()
    }

    fn bind(&mut self, socket: &mut UdpSocket, local_port: u16) -> io::Result<()> {
        *socket = UdpSocket::bind(SocketAddr::new(self.local_ip, local_port))
            .expect("couldn't bind to address");
        Ok(())
    }

    fn send_to(
        &mut self,
        socket: &mut UdpSocket,
        remote: SocketAddr,
        buffer: &[u8],
    ) -> nb::Result<(), nb::Error<io::Error>> {
        socket.send_to(buffer, remote).expect("couldn't send data");
        Ok(())
    }
}

fn main() {
    let mut tc = TftpClient::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let mut socket = tc.socket().unwrap();
    tc.bind(&mut socket, 8081).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let path = "read_from.txt";

    let packet: Vec<u8> = wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
        .unwrap()
        .into();
    tc.send_to(
        &mut socket,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 69),
        packet.as_slice(),
    )
    .unwrap();

    //necessary to add break after several error messages
    loop {
        let mut r_buf = [0; 516];
        let (number_of_bytes, src_addr) = tc.receive(&mut socket, &mut r_buf).unwrap();
        let filled_buf = &mut r_buf[..number_of_bytes];
        let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(0) => {
                println!("receive ack message");
                tc.connect(&mut socket, src_addr).unwrap();
                break;
            }

            _ => continue,
        }
    }

    let vec = tc.read_file("read_from.txt");
    let mut i = 0;
    let mut j = 512;
    let mut vec_slice: &[u8];

    loop {
        vec_slice = if vec.len() > j { &vec[i..j] } else { &vec[i..] };
        i += 512;
        j += 512;

        let block_id = 1u16;
        let packet: Vec<u8> = data(block_id, vec_slice).unwrap().into();

        loop {
            tc.send(&mut socket, packet.as_slice()).unwrap();
            let mut r_buf = [0; 516];
            let (number_of_bytes, _src_addr) = tc.receive(&mut socket, &mut r_buf).unwrap();
            let filled_buf = &mut r_buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
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
        if vec.len() <= j {
            println!("file came to end");
            break;
        }
    }
}
