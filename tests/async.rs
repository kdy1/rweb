#![cfg(not(feature = "openapi"))]

use rweb::{get, Filter, Rejection};

async fn task() -> Result<String, Rejection> {
    Ok(String::from("TEST"))
}

#[get("/")]
async fn index() -> Result<String, Rejection> {
    task().await
}

#[tokio::test]
async fn index_test() {
    let value = warp::test::request()
        .path("/")
        .reply(&index())
        .await
        .into_body();

    assert_eq!(value, b"TEST"[..]);
}

#[get("/foo")]
async fn foo() -> Result<String, Rejection> {
    task().await
}

#[tokio::test]
async fn foo_test() {
    let filter = assert_filter(foo());

    let value = warp::test::request()
        .path("/foo")
        .reply(&filter)
        .await
        .into_body();

    assert_eq!(value, b"TEST"[..]);
}

#[get("/param/{foo}")]
async fn param(foo: String) -> Result<String, Rejection> {
    println!("{}", foo); // to use it
    task().await
}

#[tokio::test]
async fn param_test() {
    let filter = assert_filter(param());

    let value = warp::test::request()
        .path("/param/param")
        .reply(&filter)
        .await
        .into_body();

    assert_eq!(value, b"TEST"[..]);
}

fn assert_filter<F: Filter>(
    f: F,
) -> impl Filter<Extract = F::Extract, Error = F::Error, Future = F::Future>
where
    F: Filter,
{
    f
}
