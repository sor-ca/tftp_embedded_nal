use embedded_nal::{SocketAddrV6, Ipv6Addr};
use std_embedded_nal::{Stack};
use std::{
    fs::File,
    io::{Read, Write},
    str::from_utf8,
};
use tftp_embedded_nal::client::TftpClient;

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
    let data = match client.read_file(
        "file2.txt",
        //&mut SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::localhost(), 69)))
        &mut remote_addr)
        {
        Ok(data) => data,
        Err(_) => panic!("can't read file"),
    };
    let mut f = File::create("write_into.txt").unwrap();
    f.write(data.as_slice()).unwrap();
    println!("{:?}", from_utf8(data.as_slice()).unwrap());

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