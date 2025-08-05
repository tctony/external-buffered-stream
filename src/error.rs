use std::sync::PoisonError;

#[derive(Debug)]
pub enum Error {
    Custom(Box<dyn std::error::Error + Send + Sync>),

    #[cfg(feature = "bincode")]
    EncodeError(bincode::error::EncodeError),
    #[cfg(feature = "bincode")]
    DecodeError(bincode::error::DecodeError),
    #[cfg(feature = "sled")]
    SledError(sled::Error),
    #[cfg(feature = "sled")]
    InvalidSledKeyFormat,

    // Failed to accquire a mutex lock
    MutexError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Custom(inner) => write!(f, "Custom error: {}", inner),

            #[cfg(feature = "bincode")]
            Error::EncodeError(e) => write!(f, "Encode error: {}", e),
            #[cfg(feature = "bincode")]
            Error::DecodeError(e) => write!(f, "Decode error: {}", e),

            #[cfg(feature = "sled")]
            Error::SledError(e) => write!(f, "Sled error: {}", e),
            #[cfg(feature = "sled")]
            Error::InvalidSledKeyFormat => write!(f, "Invalid key format"),

            Error::MutexError => write!(f, "Failed to acquire mutex lock"),
        }
    }
}

impl std::error::Error for Error {}

pub fn make_custom_error(err: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::Custom(Box::new(err))
}

#[cfg(test)]
mod tests {
    use super::make_custom_error;

    #[test]
    fn test_custom_error_display() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Test error");
        let err = make_custom_error(error);
        assert_eq!(format!("{}", err), "Custom error: Test error");
    }
}

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

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::MutexError
    }
}
