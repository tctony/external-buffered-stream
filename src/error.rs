#[derive(Debug)]
pub enum Error {
    EncodeError(bincode::error::EncodeError),
    DecodeError(bincode::error::DecodeError),
    SledError(sled::Error),
    InvalidSledKeyFormat,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::EncodeError(e) => write!(f, "Encode error: {}", e),
            Error::DecodeError(e) => write!(f, "Decode error: {}", e),
            Error::SledError(e) => write!(f, "Sled error: {}", e),
            Error::InvalidSledKeyFormat => write!(f, "Invalid key format"),
        }
    }
}

impl std::error::Error for Error {}

impl From<bincode::error::EncodeError> for Error {
    fn from(err: bincode::error::EncodeError) -> Self {
        Error::EncodeError(err)
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(err: bincode::error::DecodeError) -> Self {
        Error::DecodeError(err)
    }
}

impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        Error::SledError(err)
    }
}
