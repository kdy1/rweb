#![deny(warnings)]

use rweb::{get, Filter};
use std::{convert::Infallible, str::FromStr, time::Duration};

#[tokio::main]
async fn main() {
    // Match `/:Seconds`...
    let routes = rweb::path::param()
        // and_then create a `Future` that will simply wait N seconds...
        .and_then(sleepy);

    rweb::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

#[get("/{seconds}")]
async fn sleepy(seconds: Seconds) -> Result<impl rweb::Reply, Infallible> {
    tokio::time::delay_for(Duration::from_secs(seconds.0)).await;
    Ok(format!("I waited {} seconds!", seconds))
}

/// A newtype to enforce our maximum allowed seconds.
struct Seconds(u64);

impl FromStr for Seconds {
    type Err = ();
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.parse::<u64>().map_err(|_| ()).and_then(|num| {
            if num <= 5 {
                Ok(Seconds(num))
            } else {
                Err(())
            }
        })
    }
}