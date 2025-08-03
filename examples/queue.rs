use external_buffered_stream::create_queued_stream;
use futures::StreamExt;
use rand::seq::SliceRandom;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NumberData {
    value: i32,
}

fn create_number_stream() -> impl futures::Stream<Item = NumberData> {
    let mut rng = rand::rng();
    let mut numbers: Vec<i32> = (1..=10).collect();
    numbers.shuffle(&mut rng);
    log::info!("numbers: {:?}", numbers);
    let numbers: Vec<_> = numbers
        .into_iter()
        .map(|i| NumberData { value: i })
        .collect();

    let mut counter: i32 = 0;
    IntervalStream::new(interval(Duration::from_millis(120)))
        .take(10 as usize)
        .map(move |_| {
            counter += 1;
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
    let mut buffered_stream = create_queued_stream(number_stream)?;

    while let Some(data) = buffered_stream.next().await {
        log::info!("did process {}", data.value);

        delay(500).await;
    }

    drop(buffered_stream);
    log::info!("process stopped ");

    Ok(())
}
