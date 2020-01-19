#![deny(warnings)]

use rweb::{post, Filter, Reply};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

// TODO: Limit body size
#[post("/employees/{rate}")]
fn rate(rate: u32, #[json] mut employee: Employee) -> impl Reply {
    employee.rate = rate;
    rweb::reply::json(&employee)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    rweb::serve(rate()).run(([127, 0, 0, 1], 3030)).await
}
