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

#[derive(Default, Clone)]
struct Db {}

#[get("/use")]
fn use_db(#[data] _db: Db) -> String {
    String::new()
}

#[router("/data", services(use_db))]
fn data_param(#[data] db: Db) {}

#[tokio::test]
async fn data_param_test() {
    let value = warp::test::request()
        .path("/data/use")
        .reply(&data_param(Db::default()))
        .await;
    assert_eq!(value.status(), StatusCode::OK);
    assert_eq!(value.into_body(), b""[..]);
}

#[get("/")]
fn admin_index() -> String {
    String::new()
}

#[get("/users")]
fn admin_users() -> String {
    String::new()
}

#[router("/admin", services(admin_index, admin_users))]
#[header("X-User-Admin", "1")]
fn admin() {}
