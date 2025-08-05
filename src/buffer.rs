#[cfg(feature = "sled")]
mod sled;
#[cfg(feature = "sled")]
pub use sled::ExternalBufferSled;

#[cfg(feature = "queue")]
mod queue;
#[cfg(feature = "queue")]
pub use queue::ExternalBufferQueue;

use crate::Error;

/// The external buffer here allow us to:
///   - save items in an external perssistant storage to achieve crash save
///     for data.
///   - even with a in memory buffer, we can still implement a priority
///     queue for push and shift actions.
#[async_trait::async_trait]
pub trait ExternalBuffer<T: Sized>: Send + Sync {
    async fn push(&self, item: T) -> Result<(), Error>; // to end of buffer

    async fn shift(&self) -> Result<Option<T>, Error>; // from head of buffer
}
