use std::time::Duration;

use external_buffered_stream::{
    bincode::{Decode, Encode},
    create_external_buffered_stream,
};
use futures::stream::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

#[derive(Debug, Clone, Encode, Decode)]
struct NumberData {
    value: i32,
}

fn create_number_stream() -> impl futures::Stream<Item = NumberData> {
    let numbers: Vec<_> = (1..=10)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|i| NumberData { value: i })
        .collect();

    let mut counter: i32 = 0;
    IntervalStream::new(interval(Duration::from_millis(300)))
        .take(10 as usize)
        .map(move |_| {
            counter += 1;
            log::info!("produce {}", counter - 1);
            numbers[(counter - 1) as usize].clone()
        })
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

    let number_stream = create_number_stream();

    let buffer_dir = tempfile::Builder::new()
        .prefix("external-buffered-stream")
        .tempdir()
        .unwrap();
    let buffer_path = buffer_dir.path().to_string_lossy().to_string();

    let mut buffered_stream = create_external_buffered_stream(number_stream, buffer_path)?;

    while let Some(data) = buffered_stream.next().await {
        log::info!("did process {}", data.value);

        delay(200).await;
    }

    drop(buffered_stream);
    log::info!("process stopped ");

    Ok(())
}
