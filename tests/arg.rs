use bytes::Bytes;
use http::{Response, StatusCode};
use hyper::Body;
use rweb::{post, reply::Reply, Filter};
use serde::{Deserialize, Serialize};

struct Error {}
impl Reply for Error {
    fn into_response(self) -> Response<Body> {
        StatusCode::from_u16(500).unwrap().into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct LoginForm {
    id: String,
    password: String,
}

#[post("/json")]
fn json(#[json] body: LoginForm) -> Result<String, Error> {
    Ok(serde_json::to_string(&body).unwrap())
}

#[post("/body")]
fn body(#[body] body: Bytes) -> Result<String, Error> {
    let _ = body;
    Ok(String::new())
}

#[post("/form")]
fn form(#[form] body: LoginForm) -> Result<String, Error> {
    Ok(serde_json::to_string(&body).unwrap())
}

//#[post("/")]
//fn query(#[query] query: rweb::Json<Req>) -> Result<String, Error> {
//    Err(Error {})
//}

#[test]
fn bind() {
    rweb::serve(json().or(body()).or(form()));
}
