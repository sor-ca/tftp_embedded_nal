use std::{
    fs::File,
    io::{Read, Write},
    thread::{self, JoinHandle}
    //time::{self, Duration},
};

use tftp::{FileOperation, Message};
use std_embedded_nal::Stack;
use server::{TftpServer, RequestType};




fn main() {

    let std_stack = Stack::default();

    // create tftp client
    let mut server = TftpServer::new(
        std_stack,
    );
    //let mut handles: Vec<JoinHandle<_>> = vec![];
    println!("create server");

    loop {
        let (req_type, src_addr, filename)
            = match server.listen() {
            Ok(result)  => result,
            Err(_) => panic!("no request"),
        };
        match req_type {
            RequestType::Write =>  {
                //let handle =
                thread::spawn(move || {
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
                //handles.push(handle);
            },

            RequestType::Read =>  {
                //let handle =
                thread::spawn(move || {
                    let stack = Stack::default();
                    let mut r_server = TftpServer::new_connected(stack, &src_addr);
                    let mut vec: Vec<u8> = vec![];
                    let mut f = File::open(filename.as_str()).unwrap();
                    f.read_to_end(&mut vec).unwrap();
                    match r_server.read(src_addr, &mut vec) {
                        Ok(_)  => (),
                        Err(_) => println!("server reading error"),
                    };
                });
                //handles.push(handle);
            },
        };
    }
    //will be used after addition of timer for listening
    /*for handle in handles {
        handle.join().unwrap();
    }*/

}
