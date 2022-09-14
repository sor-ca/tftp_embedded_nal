use std::{
    fs::File,
    io::{Read, Write},
};
use std_embedded_nal::Stack;
use server::{TftpServer, RequestType};

use heapless::Vec;

fn main() {
    let std_stack = Stack::default();
    let mut server = TftpServer::new(
        std_stack,
    );
    println!("create server");
    let (req_type, src_addr, filename) = match server.listen() {
        Ok(result)  => result,
        Err(_) => panic!("no request"),
    };

    match req_type {
        RequestType::Write => match server.write(src_addr) {
            Ok(vec)  => {
                let mut f = File::create(filename.as_str()).unwrap();
                f.write(vec.as_slice()).unwrap();
            },
            Err(_) => println!("server writing error"),
        },
        RequestType::Read => {
            let mut vec: std::vec::Vec<u8> = vec![];
            let mut f = File::open(filename.as_str()).unwrap();
            f.read_to_end(&mut vec).unwrap();
            let mut h_vec: Vec<u8, {1024 * 1024}> = heapless::Vec::from_slice(&vec[..]).unwrap();

            match server.read(src_addr, &mut h_vec) {
                Ok(_)  => (),
                Err(_) => println!("server reading error"),
            }
        },
    }
}