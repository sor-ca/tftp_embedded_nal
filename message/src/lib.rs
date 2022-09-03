use ascii::AsciiStr;
use tftp::{BufAtMost512, FileOperation, Message, Mode};
use std::io;
use embedded_nal::UdpClientStack;


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

pub fn data<'b, T>(block_id: u16, buf: &'b [u8]) -> Result<Message<'b>, MyError<T>> {
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

#[derive(Debug)]
pub enum MyError<T>
where
        T: UdpClientStack,
{
    TftpErr(tftp::Error),
    UdpErr(UdpErr),
    FileErr(io::Error),
    WouldBlock,
    UdpClientStackErr(T::Error)
}

#[derive(Debug)]
pub enum UdpErr {
    BindErr,
    ConnectErr,
    SendErr,
    ReceiveErr,
}

impl<T> From<tftp::Error> for MyError<T>
where
        T: UdpClientStack,
{
    fn from(e: tftp::Error) -> Self {
        MyError::TftpErr(e)
    }
}

impl<T> From<io::Error> for MyError<T>
where
        T: UdpClientStack,
{
    fn from(e: io::Error) -> Self {
        MyError::FileErr(e)
    }
}

impl<T> From<UdpErr> for MyError<T>
where
        T: UdpClientStack,
{
    fn from(e: UdpErr) -> Self {
        MyError::UdpErr(e)
    }
}

impl<T> From<T::Error> for MyError<T>
where
        T: UdpClientStack,
{
    fn from(e: T::Error) -> Self {
        MyError::UdpClientStackErr(e)
    }
}

/*https://stackoverflow.com/a/37347504/9123725

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