use http::Error;
use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LoginForm {
    id: String,
    password: String,
}

#[get("/param/{foo}")]
fn body_after_path_param(foo: String, #[json] body: LoginForm) -> Result<String, Error> {
    assert_eq!(body.id, "TEST_ID");
    assert_eq!(body.password, "TEST_PASSWORD");
    Ok(foo)
}

#[tokio::test]
async fn test_body_after_path_param() {
    let value = warp::test::request()
        .path("/param/foo")
        .body(
            serde_json::to_vec(&LoginForm {
                id: "TEST_ID".into(),
                password: "TEST_PASSWORD".into(),
            })
            .unwrap(),
        )
        .reply(&body_after_path_param())
        .await
        .into_body();

    assert_eq!(value, b"foo"[..]);
}

#[get("/param/{foo}")]
fn path_param_after_body(#[json] body: LoginForm, foo: String) -> Result<String, Error> {
    assert_eq!(body.id, "TEST_ID");
    assert_eq!(body.password, "TEST_PASSWORD");
    Ok(foo)
}

#[tokio::test]
async fn test_path_param_after_body() {
    let value = warp::test::request()
        .path("/param/foo")
        .body(
            serde_json::to_vec(&LoginForm {
                id: "TEST_ID".into(),
                password: "TEST_PASSWORD".into(),
            })
            .unwrap(),
        )
        .reply(&path_param_after_body())
        .await
        .into_body();

    assert_eq!(value, b"foo"[..]);
}
