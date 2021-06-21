#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Item")]
struct ComponentTestReq {
    data: String,
    #[serde(skip)]
    not_schema: NotSchema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotSchema {}

#[get("/")]
fn example(_: Query<ComponentTestReq>) -> String {
    String::new()
}
