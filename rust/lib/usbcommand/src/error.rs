use super::UsbResult;
use std::fmt::Display;

#[derive(Debug)]
pub struct Mismatch<T> {
    expected: T,
    actual: T,
}

impl<T> Mismatch<T> {
    pub fn new(expected: T, actual: T) -> Self {
        Self {
            expected: expected,
            actual: actual,
        }
    }
}

#[derive(Debug)]
pub enum ProtocolError {
    ResponseSizeMismatch(Mismatch<usize>),
    TransactionIdMismatch(Mismatch<u32>),
    PacketTypeMismatch(Mismatch<u32>),
    InvalidString,
}

#[derive(Debug)]
pub enum Error {
    NoDevice,
    Transport(rusb::Error),
    Firmware(UsbResult),
    Access,
    Timeout,
    Protocol(ProtocolError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "A Usbcommand error occurred")
    }
}

impl std::convert::From<rusb::Error> for Error {
    fn from(t: rusb::Error) -> Self {
        println!("Converted usb error: {}", t);
        match t {
            rusb::Error::Timeout => Error::Timeout,
            rusb::Error::Access => Error::Access,
            rusb::Error::NoDevice => Error::NoDevice,
            _ => Error::Transport(t),
        }
    }
}

impl std::convert::From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Self {
        Error::Protocol(ProtocolError::InvalidString)
    }
}
