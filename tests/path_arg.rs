use bytes::Bytes;
use http::Error;
use rweb::{post, Filter};
use serde::{Deserialize, Serialize};

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

#[tokio::test]
async fn bind() {
    rweb::serve(json().or(body()).or(form()));
}
