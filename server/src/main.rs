use std::{
    fs::File,
    io::{Read, Write},
    thread,
    //time::{self, Duration},
};

use tftp::{FileOperation, Message};
use std_embedded_nal::Stack;
use server::TftpServer;




fn main() {

    let std_stack = Stack::default();

    // create tftp client
    let mut server = TftpServer::new(
        std_stack,
    );
    println!("create server");

    let (src_addr, mess) = match server.listen() {
        Ok(result)  => result,
        Err(_) => panic!("no request"),
    };
    let message: tftp::Message = mess[..].try_into().unwrap();
    match message {
        Message::File {
            operation: FileOperation::Write,
            ..
        } => match server.write(src_addr) {
            Ok(vec)  => {
                let mut f = File::create("file1.txt").unwrap();
                f.write(vec.as_slice()).unwrap();
            },
            Err(_) => println!("server writing error"),
        },

        Message::File {
            operation: FileOperation::Read,
            path,
            ..
        } => {
            let mut vec: Vec<u8> = vec![];
            let mut f = File::open(path.as_str()).unwrap();
            f.read_to_end(&mut vec).unwrap();
            match server.read(src_addr, &mut vec) {
                Ok(_)  => (),
                Err(_) => println!("server reading error"),
            }
        },
        //to satisfy the compiler
        _ => (),
    };
}
