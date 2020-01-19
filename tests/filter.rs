use rweb::*;
use std::{net::SocketAddr, str::FromStr};

fn host_header<T: FromStr + Send + 'static>(
) -> impl Clone + Filter<Extract = (T,), Error = Rejection> {
    rweb::header::<T>("host")
}

fn accept_all_header() -> impl Clone + Filter<Extract = (), Error = Rejection> {
    rweb::header::exact("accept", "*/*")
}

#[get("/")]
fn handler_guard(#[filter = "accept_all_header"] _header: ()) -> String {
    String::new()
}

#[tokio::test]
async fn handler_guard_test() {
    let value = warp::test::request()
        .path("/")
        .reply(&handler_guard())
        .await
        .into_body();

    assert_eq!(value, b""[..]);
}

#[get("/")]
fn handler_value(#[filter = "host_header"] addr: SocketAddr) -> String {
    addr.to_string()
}

#[tokio::test]
async fn handler_value_test() {
    let value = warp::test::request()
        .path("/")
        .reply(&handler_value())
        .await
        .into_body();

    assert_eq!(value, b"127.0.0.1"[..]);
}

#[get("/")]
fn handler_mixed(
    #[filter = "accept_all_header"] _header: (),
    #[filter = "host_header"] addr: SocketAddr,
) -> String {
    addr.to_string()
}

#[tokio::test]
async fn handler_mixed_test() {
    let value = warp::test::request()
        .path("/")
        .reply(&handler_mixed())
        .await
        .into_body();

    assert_eq!(value, b"127.0.0.1"[..]);
}
