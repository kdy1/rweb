#![deny(warnings)]

use rweb::Filter;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // POST /employees/:rate  {"name":"Sean","rate":2}
    let promote = rweb::post()
        .and(rweb::path("employees"))
        .and(rweb::path::param::<u32>())
        // Only accept bodies smaller than 16kb...
        .and(rweb::body::content_length_limit(1024 * 16))
        .and(rweb::body::json())
        .map(|rate, mut employee: Employee| {
            employee.rate = rate;
            rweb::reply::json(&employee)
        });

    rweb::serve(promote).run(([127, 0, 0, 1], 3030)).await
}
