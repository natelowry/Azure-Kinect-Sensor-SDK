use std::fmt::Display;
use crate::error::Mismatch;

#[derive(Debug)]
pub enum Error {
    Load(std::io::Error),
    Version(Mismatch<super::Version>),
    IncompatibleInterface,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Error::Load(x)
    }
}
