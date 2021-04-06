#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "TestStruct")]
pub struct TestStruct {
    ///a description
    #[schema(example = "an example")]
    d1: String,
    #[schema(example = "\"an example\"")]
    #[schema(description = "a description")]
    d2: String,
    #[schema(example = "\"an example\"", description = "a description")]
    d3: String,
}

#[get("/")]
fn test_r(_: Query<TestStruct>) -> String {
    String::new()
}

#[test]
fn test_description_example() {
    let (spec, _) = openapi::spec().build(|| test_r());
    let schema = match spec
        .components
        .as_ref()
        .unwrap()
        .schemas
        .get("TestStruct")
        .unwrap()
    {
        openapi::ObjectOrReference::Object(s) => s,
        _ => panic!(),
    };
    println!("{}", serde_yaml::to_string(&schema).unwrap());
    for (_, p) in &schema.properties {
        assert_eq!(p.description, "a description");
        assert_eq!(
            p.example,
            Some(serde_json::from_str("\"an example\"").unwrap())
        );
    }
}
