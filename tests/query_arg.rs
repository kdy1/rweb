use rweb::*;

#[get("/")]
fn use_query(#[query] qs: String) -> String {
    qs
}

#[tokio::test]
async fn query_str() {
    let value = warp::test::request()
        .path("/?q=s")
        .reply(&use_query())
        .await
        .into_body();

    assert_eq!(value, b"q=s"[..]);
}
