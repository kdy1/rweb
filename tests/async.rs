use http::Error;
use rweb::{get, serve};

async fn task() -> Result<String, Error> {
    Ok(String::from("TEST"))
}

#[get("/")]
async fn index() -> Result<String, Error> {
    task().await
}

#[get("/foo")]
async fn foo() -> Result<String, Error> {
    task().await
}

#[get("/param/{foo}")]
async fn param(foo: String) -> Result<String, Error> {
    println!("{}", foo); // to use it
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

#[tokio::test]
async fn foo_test() {
    let value = warp::test::request()
        .path("/")
        .reply(&foo())
        .await
        .into_body();

    assert_eq!(value, b"TEST"[..]);
}

#[tokio::test]
async fn param_test() {
    let value = warp::test::request()
        .path("/param/param")
        .reply(&param())
        .await
        .into_body();

    assert_eq!(value, b"TEST"[..]);
}
