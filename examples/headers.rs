#![deny(warnings)]

use rweb::*;
use std::net::SocketAddr;

/// For this example, we assume no DNS was used,
/// so the Host header should be an address.
///
/// Match when we get `accept: */*` exactly.
#[get("/")]
fn routes(#[header(accept = "*/*")] _guard: (), #[header = "host"] host: SocketAddr) -> String {
    format!("accepting stars on {}", host)
}

/// Create a server that requires header conditions:
///
/// - `Host` is a `SocketAddr`
/// - `Accept` is exactly `*/*`
///
/// If these conditions don't match, a 404 is returned.
#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    rweb::serve(routes()).run(([127, 0, 0, 1], 3030)).await;
}
