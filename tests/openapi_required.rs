#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "TestStruct")]
struct TestStruct {
    afield: String,
    optfield: Option<String>,
}

#[get("/")]
fn test_struct_r(_: Query<TestStruct>) -> String {
    String::new()
}

#[test]
fn test_struct() {
    let (spec, _) = openapi::spec().build(|| test_struct_r());
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
    assert!(schema.required.contains(&Cow::Borrowed("afield")));
    assert!(!schema.required.contains(&Cow::Borrowed("optfield")));
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "TestEnum")]
enum TestEnum {
    AThing {
        afield: String,
    },
    MaybeThing {
        optfield: Option<String>,
    },
    TwoThings {
        afield: String,
        optfield: Option<String>,
    },
}

#[get("/")]
fn test_enum_r(_: Query<TestEnum>) -> String {
    String::new()
}

#[test]
fn test_enum() {
    let (spec, _) = openapi::spec().build(|| test_enum_r());
    let schema = match spec
        .components
        .as_ref()
        .unwrap()
        .schemas
        .get("TestEnum")
        .unwrap()
    {
        openapi::ObjectOrReference::Object(s) => s,
        _ => panic!(),
    };
    println!("{}", serde_yaml::to_string(&schema).unwrap());
    for variant in &schema.one_of {
        match variant {
            openapi::ObjectOrReference::Object(vs) => {
                if vs.properties.contains_key(&Cow::Borrowed("afield")) {
                    assert!(vs.required.contains(&Cow::Borrowed("afield")));
                }
                if vs.properties.contains_key(&Cow::Borrowed("optfield")) {
                    assert!(!vs.required.contains(&Cow::Borrowed("optfield")));
                }
            }
            _ => panic!(),
        }
    }
}
