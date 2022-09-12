use std::{
    fs::File,
    io::{Read, Write},
    thread::{self, JoinHandle}
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
    let mut handles: Vec<JoinHandle<_>> = vec![];
    println!("create server");

    loop {
        let (src_addr, mess) = match server.listen() {
            Ok(result)  => result,
            Err(_) => panic!("no request"),
        };
        let message: tftp::Message = mess[..].try_into().unwrap();
        match message {
            Message::File {
                operation: FileOperation::Write,
                ..
            } => {
                let mut handle = thread::spawn(move || {
                    let stack = Stack::default();
                    let mut w_server = TftpServer::new_connected(stack, &src_addr);
                    match w_server.write(src_addr) {
                        Ok(vec)  => {
                            let mut f = File::create("file1.txt").unwrap();
                            f.write(vec.as_slice()).unwrap();
                        },
                        Err(_) => println!("server writing error"),
                    };
                });
                handles.push(handle);
            },

            Message::File {
                operation: FileOperation::Read,
                path,
                ..
            } => {
                let handle = thread::spawn(move || {
                    let stack = Stack::default();
                    let mut r_server = TftpServer::new_connected(stack, &src_addr);
                    let mut vec: Vec<u8> = vec![];
                    let mut f = File::open(path.as_str()).unwrap();
                    f.read_to_end(&mut vec).unwrap();
                    match r_server.read(src_addr, &mut vec) {
                        Ok(_)  => (),
                        Err(_) => println!("server reading error"),
                    };
                });
                handles.push(handle);
            },
            //to satisfy the compiler
            _ => (),
        };
    }
    //will be used after addition of timer for listening
    for handle in handles {
        handle.join().unwrap();
    }

}
