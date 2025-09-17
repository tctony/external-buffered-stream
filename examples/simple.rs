use external_buffered_stream::{
    bincode::{Decode, Encode},
    create_external_buffered_stream, Error,
};
use futures::{stream, StreamExt};
use std::time::Duration;
use tokio::time::{interval, timeout};
use tokio_stream::wrappers::IntervalStream;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct NumberData {
    value: i32,
}

fn create_number_stream() -> impl futures::Stream<Item = NumberData> {
    let mut counter: i32 = 0;
    IntervalStream::new(interval(Duration::from_millis(120)))
        .take(10 as usize)
        .map(move |_| {
            counter += 1;
            log::info!("produce {}", counter);
            NumberData { value: counter }
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

    let buffer_dir = tempfile::Builder::new()
        .prefix("external-buffered-stream")
        .tempdir()
        .unwrap();
    let buffer_path = buffer_dir.path().to_string_lossy().to_string();

    let handle = tokio::spawn(async move {
        timeout(Duration::from_secs(3), async move {
            let number_stream = create_number_stream();
            let mut buffered_stream = create_external_buffered_stream(number_stream, buffer_path)?;

            while let Some(data) = buffered_stream.next().await {
                log::info!("flow1 did process {}", data.value);

                delay(500).await;
            }

            drop(buffered_stream);
            log::info!("flow1 stopped");

            Ok::<(), Error>(())
        })
        .await
    });
    match handle.await? {
        Ok(Ok(_)) => log::info!("flow1 finished"),
        Ok(Err(err)) => log::error!("flow1 error: {}", err),
        Err(elpased) => log::info!("flow1 timeout: {}", elpased),
    }

    let buffer_path = buffer_dir.path().to_string_lossy().to_string();
    let result = tokio::spawn(async move {
        let empty_stream = stream::empty::<NumberData>();
        let mut new_buffered_stream = create_external_buffered_stream(empty_stream, buffer_path)?;

        while let Some(data) = new_buffered_stream.next().await {
            log::info!("flow2 did process {}", data.value,);

            delay(500).await;
        }
        Ok::<(), Error>(())
    })
    .await;

    log::info!("result of flow2: {:?}", result);

    Ok(())
}
