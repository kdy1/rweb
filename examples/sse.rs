use rweb::{filters::sse::Event, get, Reply};
use std::{convert::Infallible, time::Duration};
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

// create server-sent event
fn sse_counter(counter: u64) -> Result<Event, Infallible> {
    Ok(Event::default().data(counter.to_string()))
}

#[get("/ticks")]
fn ticks() -> impl Reply {
    let mut counter: u64 = 0;
    // create server event source
    let event_stream = IntervalStream::new(interval(Duration::from_secs(1))).map(move |_| {
        counter += 1;
        sse_counter(counter)
    });
    // reply using server-sent events
    rweb::sse::reply(event_stream)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    rweb::serve(ticks()).run(([127, 0, 0, 1], 3030)).await;
}
