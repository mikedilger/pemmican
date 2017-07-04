
use std::net::AddrParseError;
use hyper::Error as HyperError;

pub enum Error {
    Hyper(HyperError),
    AddrParse(AddrParseError),
}

impl From<HyperError> for Error {
    fn from(e: HyperError) -> Error {
        Error::Hyper(e)
    }
}

impl From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Error {
        Error::AddrParse(e)
    }
}
