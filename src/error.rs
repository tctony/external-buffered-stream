#[derive(Debug)]
pub enum Error {
    Unknown,
    #[cfg(feature = "bincode")]
    EncodeError(bincode::error::EncodeError),
    #[cfg(feature = "bincode")]
    DecodeError(bincode::error::DecodeError),
    #[cfg(feature = "sled")]
    SledError(sled::Error),
    #[cfg(feature = "sled")]
    InvalidSledKeyFormat,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Unknown => write!(f, "Unknown error"),
            #[cfg(feature = "bincode")]
            Error::EncodeError(e) => write!(f, "Encode error: {}", e),
            #[cfg(feature = "bincode")]
            Error::DecodeError(e) => write!(f, "Decode error: {}", e),
            #[cfg(feature = "sled")]
            Error::SledError(e) => write!(f, "Sled error: {}", e),
            #[cfg(feature = "sled")]
            Error::InvalidSledKeyFormat => write!(f, "Invalid key format"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "bincode")]
impl From<bincode::error::EncodeError> for Error {
    fn from(err: bincode::error::EncodeError) -> Self {
        Error::EncodeError(err)
    }
}

#[cfg(feature = "bincode")]
impl From<bincode::error::DecodeError> for Error {
    fn from(err: bincode::error::DecodeError) -> Self {
        Error::DecodeError(err)
    }
}

#[cfg(feature = "sled")]
impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        Error::SledError(err)
    }
}
