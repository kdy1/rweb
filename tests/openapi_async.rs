#![cfg(feature = "openapi")]

use indexmap::IndexMap;
use rweb::{openapi::ResponseEntity, *};
use std::borrow::Cow;

async fn task() -> String {
    String::from("TEST")
}

#[derive(Debug, Schema)]
struct CustomResp {
    value: String,
}

impl ResponseEntity for CustomResp {
    fn describe_responses() -> IndexMap<Cow<'static, str>, rweb::openapi::Response> {
        Default::default()
    }
}

#[get("/")]
async fn custom() -> CustomResp {
    CustomResp {
        value: task().await,
    }
}

#[tokio::test]
async fn custom_test() {
    let value = warp::test::request()
        .path("/")
        .reply(&custom())
        .await
        .into_body();

    assert_eq!(value, b"TEST"[..]);
}
