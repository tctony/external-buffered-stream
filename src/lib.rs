mod buffer;
mod error;
mod serde;
mod runtime;

pub use buffer::*;
pub use error::*;
pub use serde::*;

use std::{
    marker::PhantomData,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

use futures::{FutureExt, SinkExt, Stream, StreamExt, channel::mpsc};

pub struct ExternalBufferedStream<T, B, S>
where
    T: Send,
    B: ExternalBuffer<T>,
    S: Stream<Item = T>,
{
    buffer: Arc<B>,
    _source: PhantomData<S>,
    notify: mpsc::UnboundedReceiver<()>,
    stop_flag: Arc<AtomicBool>,
}

impl<T, B, S> ExternalBufferedStream<T, B, S>
where
    T: Send,
    B: ExternalBuffer<T> + 'static,
    S: Stream<Item = T> + Send + 'static,
{
    pub fn new(source: S, buffer: B) -> Self {
        let source = Box::pin(source);

        let buffer = Arc::new(buffer);
        let buffer_clone = buffer.clone();

        let (notify_tx, notify_rx) = mpsc::unbounded::<()>();

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let handle_source = async move {
            let mut source = source;
            let mut notify_tx = notify_tx;
            while let Some(item) = source.next().await {
                match buffer_clone.push(item).await {
                    Ok(()) => match notify_tx.send(()).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Failed to notify: {:?}", e);
                            break;
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to push item to buffer: {:?}", e);
                        break;
                    }
                }
            }
            log::info!("Source stream is ended");
            stop_flag_clone.store(true, Ordering::SeqCst);
            _ = notify_tx.send(())
        };
        runtime::spawn(handle_source);

        ExternalBufferedStream {
            buffer,
            _source: PhantomData,
            notify: notify_rx,
            stop_flag,
        }
    }
}

impl<T, B, S> Stream for ExternalBufferedStream<T, B, S>
where
    T: Send,
    B: ExternalBuffer<T> + 'static,
    S: Stream<Item = T> + Send + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // S is PhantomData, so here is safe to get mut
        let this = unsafe { self.get_unchecked_mut() };

        loop {
            match this.buffer.shift().poll_unpin(ctx) {
                Poll::Ready(next) => match next {
                    Ok(Some(item)) => return Poll::Ready(Some(item)),
                    Ok(None) => {
                        let mut wait = (&mut this.notify).next();
                        match wait.poll_unpin(ctx) {
                            Poll::Ready(_) => {
                                if this.stop_flag.load(Ordering::SeqCst) {
                                    break Poll::Ready(None);
                                } else {
                                    continue;
                                }
                            }
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    Err(err) => {
                        log::error!("poll external buffer error: {}", err);
                        return Poll::Ready(None);
                    }
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(feature = "default")]
pub fn create_external_buffered_stream<T, S, P>(
    stream: S,
    path: P,
) -> Result<ExternalBufferedStream<T, ExternalBufferSled, S>, Error>
where
    T: ExternalBufferSerde + Send + 'static,
    S: Stream<Item = T> + Send + Sync + 'static,
    P: AsRef<std::path::Path>,
{
    Ok(ExternalBufferedStream::new(
        stream,
        ExternalBufferSled::new(path)?,
    ))
}

#[cfg(feature = "queue")]
pub fn create_queued_stream<T, S>(
    stream: S,
) -> Result<ExternalBufferedStream<T, ExternalBufferQueue<T>, S>, Error>
where
    T: Ord + Send + 'static,
    S: Stream<Item = T> + Send + Sync + 'static,
{
    Ok(ExternalBufferedStream::new(
        stream,
        ExternalBufferQueue::new(),
    ))
}
