//use std::path::PathBuf;
//use displaydoc::Display;
use ascii::AsciiStr;
use nb::{Result, Error};
use tftp::{BufAtMost512, FileOperation, Message, Mode};
use std::io;

/*pub fn wrq<'b>(path: &'b AsciiStr, octet_mode: bool) -> Result<Message<'b>, tftp::Error> {
    //Need to correct because this is std::fs function, I haven't found no_std equivalent,
    if !PathBuf::from(path.as_str()).exists() {
        return Err(nb::Error::Other(tftp::Error::NoPath));
    }

    Ok(Message::File {
        operation: FileOperation::Write,
        path,
        mode: if octet_mode {
            Mode::Binary
        } else {
            Mode::NetAscii
        },
    })
}*/

pub fn wrq<'b>(path: &'b AsciiStr, octet_mode: bool) -> Message<'b> {
    //Need to correct because this is std::fs function, I haven't found no_std equivalent,
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

// path - need to check if the file is available
/*pub fn rrq<'b>(path: &'b AsciiStr, octet_mode: bool) -> Result<Message<'b>, tftp::Error> {
    //Need to correct because this is std::fs function, I haven't found no_std equivalent
    if !PathBuf::from(path.as_str()).exists() {
        return Err(nb::Error::Other(tftp::Error::NoPath));
    }

    Ok(Message::File {
        operation: FileOperation::Read,
        path,
        mode: if octet_mode {
            Mode::Binary
        } else {
            Mode::NetAscii
        },
    })
}*/

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

pub fn data<'b>(block_id: u16, buf: &'b [u8]) -> Result<Message<'b>, tftp::Error> {
    let buf = BufAtMost512::try_from(buf);
    match buf {
        Ok(data) => Ok(Message::Data(block_id, data)),
        Err(e) => Err(nb::Error::Other(tftp::Error::BufferTooLarge(e.0))),
    }
}

//I don't know what to with lifetime in this case.
//It is not necessary because there is no references but the compiler fails
pub fn ack<'b>(block_id: u16) -> Message<'b> {
    //I can't imagine what fail may occur
    Message::Ack(block_id)
}

pub fn error<'b>(block_id: u16, error_message: &'b AsciiStr) -> Message<'b> {
    //I can't imagine what fail may occur
    Message::Error(block_id, error_message)
}

//i don't understand exactly how many variants of errors are necessary
//for example, we need to add Error::IncorrectPath or something like that

//#[derive(Debug, Display)]
#[derive(Debug)]
pub enum MyError {
    TftpErr(tftp::Error),
    UdpErr(UdpErr),
    FileErr(io::Error),
    WouldBlock,
    //Timeout,
}

#[derive(Debug)]
pub enum UdpErr {
    BindErr,
    ConnectErr,
    SendErr,
    ReceiveErr,
}

impl From<tftp::Error> for MyError {
    fn from(e: tftp::Error) -> Self {
        MyError::TftpErr(e)
    }
}

impl From<io::Error> for MyError {
    fn from(e: io::Error) -> MyError {
        MyError::FileErr(e)
    }
}

impl From<UdpErr> for MyError {
    fn from(e: UdpErr) -> MyError {
        MyError::UdpErr(e)
    }
}

impl From<nb::Error<tftp::Error>> for MyError {
    fn from(e: nb::Error<tftp::Error>) -> MyError {
        match e {
            Error::Other(err) => MyError::TftpErr(err),
            Error::WouldBlock => MyError::WouldBlock,
        }
    }
}
