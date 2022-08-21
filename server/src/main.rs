use std::net::UdpSocket;
use std::str::from_utf8;
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8080").expect("couldn't bind to address");
    loop {
        let mut buf = [0; 516];
        let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("didn't receive data");
        dbg!(src_addr);
        let filled_buf = &mut buf[..number_of_bytes];
        dbg!(from_utf8(&filled_buf).unwrap());
        //let message_text = from_utf8(&buf[4..]).expect("can't read message");
        //println!("server receives {}", message_text);
        socket.send_to(&filled_buf, src_addr).expect("couldn't send data");
    }    
}

