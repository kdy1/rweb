use rweb::*;

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[get("/mul/{a}/{b}")]
fn mul(a: usize, b: usize) -> String {
    (a * b).to_string()
}

#[router("/math", services(sum, mul))]
struct MathRouter;

#[tokio::test]
async fn router() {
    let filter = MathRouter();

    let value = warp::test::request()
        .path("/math/sum/1/2")
        .reply(&filter)
        .await
        .into_body();
    assert_eq!(value, b"3"[..]);
}

#[get("/no-arg")]
fn no_arg() -> String {
    String::new()
}

#[router("/math/complex", services(sum, mul, no_arg))]
struct Complex;

#[tokio::test]
async fn complex_router() {
    let filter = Complex();

    let value = warp::test::request()
        .path("/math/complex/sum/1/2")
        .reply(&filter)
        .await
        .into_body();
    assert_eq!(value, b"3"[..]);
}
