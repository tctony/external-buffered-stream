use bincode::{Decode, Encode};
use external_buffered_stream::{create_external_buffered_stream, Error};
use futures::{stream, StreamExt};
use std::time::Duration;
use tokio::{select, sync::oneshot, task::yield_now};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct NumberData {
    value: i32,
}

fn create_number_stream() -> impl futures::Stream<Item = NumberData> {
    let numbers: Vec<_> = (1..=10).map(|i| NumberData { value: i }).collect();
    stream::iter(numbers)
}

async fn delay(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_millis()
        .try_init();

    let buffer_dir = tempfile::Builder::new()
        .prefix("external-buffered-stream")
        .tempdir()
        .unwrap();
    let buffer_path = buffer_dir.path().to_string_lossy().to_string();

    let (stop_tx, stop_rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let number_stream = create_number_stream();
        let mut buffered_stream = create_external_buffered_stream(number_stream, buffer_path)?;

        while let Some(data) = buffered_stream.next().await {
            log::info!("did process {}", data.value);

            delay(500).await;
        }

        drop(buffered_stream);
        log::info!("process stopped because of timeout"); // why this line is after process 10

        let _ = stop_tx.send(());

        Ok::<(), Error>(())
    });

    select! {
        _ = handle => {
            unreachable!()
        },
        _ = delay(3000) => {
            log::info!("timeout");
        }
    };

    yield_now().await;

    let _ = stop_rx.await;

    let buffer_path = buffer_dir.path().to_string_lossy().to_string();
    let result = tokio::spawn(async move {
        let empty_stream = stream::empty::<NumberData>();
        let mut new_buffered_stream = create_external_buffered_stream(empty_stream, buffer_path)?;

        while let Some(data) = new_buffered_stream.next().await {
            log::info!("did process {}", data.value,);

            delay(500).await;
        }
        Ok::<(), Error>(())
    })
    .await;
    log::info!("{:?}", result);

    Ok(())
}
