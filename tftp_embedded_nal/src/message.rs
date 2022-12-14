/*
Error Codes

   Value     Meaning

   0         Not defined, see error message (if any).
   1         File not found.
   2         Access violation.
   3         Disk full or allocation exceeded.
   4         Illegal TFTP operation.
   5         Unknown transfer ID.
   6         File already exists.
   7         No such user.
 */

use ascii::AsciiStr;
use std::io;
use embedded_nal::{UdpClientStack, UdpFullStack};
use embedded_nal::nb;
use tftp::{BufAtMost512, FileOperation, Message, Mode, Operation};
use heapless::Vec;

//impl <'b, const N: usize> From <Message<'b>> for Vec<u8, N> {
    //fn from(message: Message<'b>) -> Self {
pub fn to_heapless<'b> (message: Message<'b>) -> Vec<u8, 516> {
    match message {
        Message::File {
            operation,
            path,
            mode,
        } => {
            let mode: &'static AsciiStr = mode.into();
            let mut buf: Vec<u8, 516> = Vec::new();

            buf.extend_from_slice(&i16::to_be_bytes(Operation::from(operation) as i16)).unwrap();
            buf.extend_from_slice(path.as_bytes()).unwrap();
            buf.push(0).unwrap();
            buf.extend_from_slice(mode.as_bytes()).unwrap();
            buf.push(0).unwrap();

            buf
        }
        Message::Data(block_id, data) => {
            let mut buf: Vec<u8, 516> = Vec::new();

            buf.extend_from_slice(&i16::to_be_bytes(Operation::Data as i16)).unwrap();
            buf.extend_from_slice(&u16::to_be_bytes(block_id)).unwrap();
            buf.extend_from_slice(data.as_ref()).unwrap();

            buf
        }
        Message::Ack(block_id) => {
            let mut buf: Vec<u8, 516> = Vec::new();

            buf.extend_from_slice(&i16::to_be_bytes(Operation::Ack as i16)).unwrap();
            buf.extend_from_slice(&u16::to_be_bytes(block_id)).unwrap();

            buf
        }
        Message::Error(block_id, error) => {
            let mut buf: Vec<u8, 516> = Vec::new();

            buf.extend_from_slice(&i16::to_be_bytes(Operation::Error as i16)).unwrap();
            buf.extend_from_slice(&u16::to_be_bytes(block_id)).unwrap();
            buf.extend_from_slice(error.as_bytes()).unwrap();
            buf.push(0).unwrap();

            buf
        }
    }
}


pub fn wrq<'b>(path: &'b AsciiStr, octet_mode: bool) -> Message<'b> {
    Message::File {
        operation: FileOperation::Write,
        path,
        mode: if octet_mode {
            Mode::Binary
        } else {
            Mode::NetAscii
        },
    }
}

pub fn rrq<'b>(path: &'b AsciiStr, octet_mode: bool) -> Message<'b> {
    Message::File {
        operation: FileOperation::Read,
        path,
        mode: if octet_mode {
            Mode::Binary
        } else {
            Mode::NetAscii
        },
    }
}

pub fn data<'b, T: UdpClientStack + UdpFullStack>(block_id: u16, buf: &'b [u8]) -> Result<Message<'b>, MyError<T>> {
    let buf = BufAtMost512::try_from(buf);
    match buf {
        Ok(data) => Ok(Message::Data(block_id, data)),
        Err(e) => Err(MyError::TftpErr(tftp::Error::BufferTooLarge(e.0))),
    }
}

pub fn ack<'b>(block_id: u16) -> Message<'b> {
    Message::Ack(block_id)
}

pub fn error<'b>(block_id: u16, error_message: &'b AsciiStr) -> Message<'b> {
    Message::Error(block_id, error_message)
}

pub enum MyError<T>
where T: UdpClientStack + UdpFullStack,
{
    TftpErr(tftp::Error),
    UdpErr(UdpErr<T>),
    FileErr(io::Error),
    WouldBlock,
}

#[derive(Debug)]
pub enum UdpErr<T>
where T: UdpClientStack + UdpFullStack
{
    BindErr(<T>::Error),
    ConnectErr(<T>::Error),
    SendErr(nb::Error<<T>::Error>),
    ReceiveErr(nb::Error<<T>::Error>),
}

impl<T> From<tftp::Error> for MyError<T>
where T: UdpClientStack + UdpFullStack,
{
    fn from(e: tftp::Error) -> Self {
        MyError::TftpErr(e)
    }
}

impl<T> From<io::Error> for MyError<T>
where T: UdpClientStack + UdpFullStack,
{
    fn from(e: io::Error) -> Self {
        MyError::FileErr(e)
    }
}


/* https://stackoverflow.com/a/37347504/9123725

#[derive(Debug)]
pub enum MyError2<T>
where
       T: UdpClientStack,
{
    // TftpErr(tftp::Error),
    // UdpErr(UdpErr),
    // FileErr(io::Error),
    WouldBlock,
    UdpClientStackErr(T),
}

impl<T: UdpClientStack<Error = T>> MyError<T> {
    fn from_udp_stack_error(e: T::Error) -> Self {
        MyError::UdpClientStackErr(e)
    }
}*/
