use http::StatusCode;
use rweb::*;

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[get("/mul/{a}/{b}")]
fn mul(a: usize, b: usize) -> String {
    (a * b).to_string()
}

#[get("/no-arg")]
fn no_arg() -> String {
    String::new()
}

#[tokio::test]
async fn math_test() {
    #[router("/math", services(sum, mul))]
    fn math() {}

    let value = warp::test::request()
        .path("/math/sum/1/2")
        .reply(&math())
        .await;
    assert_eq!(value.status(), StatusCode::OK);
    assert_eq!(value.into_body(), b"3"[..]);
}

#[tokio::test]
async fn arg_cnt_test() {
    #[router("/math/complex", services(sum, mul, no_arg))]
    fn arg_cnt() {}

    let value = warp::test::request()
        .path("/math/complex/sum/1/2")
        .reply(&arg_cnt())
        .await;
    assert_eq!(value.status(), StatusCode::OK);
    assert_eq!(value.into_body(), b"3"[..]);
}
