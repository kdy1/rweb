use http::StatusCode;
use rweb::*;

#[get("/")]
fn ret_accept(#[header = "accept"] accept: String) -> String {
    accept
}

#[tokio::test]
async fn ret_accept_test() {
    let value = warp::test::request()
        .path("/")
        .header("accept", "foo")
        .reply(&ret_accept())
        .await
        .into_body();
    assert_eq!(value, b"foo"[..]);
}

#[get("/")]
#[header("X-AuthUser", "test-uid")]
fn guard() -> String {
    unreachable!()
}

#[tokio::test]
async fn guard_test() {
    let value = warp::test::request().path("/").reply(&guard()).await;

    assert_eq!(value.status(), StatusCode::BAD_REQUEST);
}
