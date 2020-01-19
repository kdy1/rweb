use rweb::*;

#[get("/")]
fn ret_accept(#[header = "accept"] accept: String) -> String {
    accept
}

#[tokio::test]
async fn complex_router() {
    let value = warp::test::request()
        .path("/")
        .header("accept", "foo")
        .reply(&ret_accept())
        .await
        .into_body();
    assert_eq!(value, b"foo"[..]);
}
