use futures::StreamExt;
use rweb::{sse::ServerSentEvent, Filter};
use std::{convert::Infallible, time::Duration};
use tokio::time::interval;

// create server-sent event
fn sse_counter(counter: u64) -> Result<impl ServerSentEvent, Infallible> {
    Ok(rweb::sse::data(counter))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let routes = rweb::path("ticks").and(rweb::get()).map(|| {
        let mut counter: u64 = 0;
        // create server event source
        let event_stream = interval(Duration::from_secs(1)).map(move |_| {
            counter += 1;
            sse_counter(counter)
        });
        // reply using server-sent events
        rweb::sse::reply(event_stream)
    });

    rweb::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
