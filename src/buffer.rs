#[cfg(feature = "sled")]
mod sled;

#[cfg(feature = "sled")]
pub use sled::ExternalBufferSled;

use crate::{Error, ExternalBufferSerde};

/// The external buffer here allow us to:
///   - save items in an external perssistant storage to achieve crash save
///     for data.
///   - even with a in memory buffer, we can still implement a priority
///     queue for push and shift actions.
pub trait ExternalBuffer<T: ExternalBufferSerde> {
    fn push(&self, item: T) -> Result<(), Error>; // to end of buffer

    fn shift(&self) -> Result<Option<T>, Error>; // from head of buffer
}
