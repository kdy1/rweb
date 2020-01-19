use http::{Response, StatusCode};
use hyper::Body;
use rweb::{get, reply::Reply, Filter};

struct Error {}
impl Reply for Error {
    fn into_response(self) -> Response<Body> {
        StatusCode::from_u16(500).unwrap().into_response()
    }
}

#[get("/")]
fn index() -> Result<String, Error> {
    Err(Error {})
}

#[get("/foo")]
fn foo() -> Result<String, Error> {
    Ok(String::new())
}

#[get("/param/{foo}")]
fn param(foo: String) -> Result<String, Error> {
    Ok(foo)
}

#[get("/param/{v}")]
fn param_typed(v: u32) -> Result<String, Error> {
    Ok(v.to_string())
}

#[test]
fn app_service() {
    rweb::serve(index().or(foo()).or(param()).or(param_typed()));
}
