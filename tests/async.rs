use http::{Response, StatusCode};
use hyper::Body;
use rweb::{get, reply::Reply, serve, Filter};

struct Error {}
impl Reply for Error {
    fn into_response(self) -> Response<Body> {
        StatusCode::from_u16(500).unwrap().into_response()
    }
}

async fn task() -> Result<String, Error> {
    Ok(String::new())
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

#[test]
fn bind() {
    serve(index().or(foo()).or(param()));
}
