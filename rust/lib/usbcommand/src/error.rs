use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    NoDevice,
    Fail,
    Access,
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
            rusb::Error::Success => Error::Fail,
            rusb::Error::Io => Error::Fail,
            rusb::Error::InvalidParam => panic!("Unexpected usb error"),
            rusb::Error::Access => Error::Access,
            rusb::Error::NoDevice => Error::NoDevice,
            rusb::Error::NotFound => Error::Fail,
            rusb::Error::Busy => Error::Fail,
            rusb::Error::Timeout => Error::Fail,
            rusb::Error::Overflow => Error::Fail,
            rusb::Error::Pipe => Error::Fail,
            rusb::Error::Interrupted => panic!("Unexpected usb error: Interrupted"),
            rusb::Error::NoMem => panic!("Unexpected usb error: NoMem"),
            rusb::Error::NotSupported => panic!("Unexpected usb error: NotSupported"),
            rusb::Error::Other => panic!("Unexpected usb error: Other"),
        }
    }
}