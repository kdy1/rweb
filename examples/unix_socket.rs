#![deny(warnings)]

use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut listener = UnixListener::bind("/tmp/warp.sock").unwrap();
    let incoming = listener.incoming();
    rweb::serve(rweb::fs::dir("examples/dir"))
        .run_incoming(incoming)
        .await;
}
