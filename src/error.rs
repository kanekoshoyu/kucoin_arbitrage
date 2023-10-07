use std::error;
use std::fmt;
use std::io;
use toml;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    TomlError(toml::de::Error),
    // Add more error variants as needed
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref err) => write!(f, "IO error: {}", err),
            Error::TomlError(ref err) => write!(f, "TOML error: {}", err),
            // Add more error variants as needed
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::IoError(ref err) => Some(err),
            Error::TomlError(ref err) => Some(err),
            // Add more error variants as needed
        }
    }
}
