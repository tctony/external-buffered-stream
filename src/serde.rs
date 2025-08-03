#[cfg(feature = "bincode")]
pub mod bincode;

use crate::Error;

/// Convert object into data that can saved in external buffer and vice versa
/// This trait is not necessary if your external use a staroge like sqlite, in
/// which case data store and retrieve without serde.
pub trait ExternalBufferSerde: Sized {
    fn into_external_buffer(self) -> Result<Vec<u8>, Error>;

    fn from_external_buffer(value: &[u8]) -> Result<Self, Error>;
}
