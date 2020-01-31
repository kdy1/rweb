#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[test]
fn struct_field() {
    #[derive(Debug, Serialize, Deserialize, Schema)]
    struct Data {
        #[serde(rename = "result")]
        data: String,
    }

    #[get("/")]
    fn index(_: Query<Data>) -> String {
        String::new()
    }

    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("result"));
}

#[test]
fn struct_rename_all() {
    #[derive(Debug, Serialize, Deserialize, Schema)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        data_msg: String,
    }

    #[get("/")]
    fn index(_: Query<Data>) -> String {
        String::new()
    }

    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("dataMsg"));
}

#[test]
fn clike_enum() {
    #[derive(Debug, Serialize, Deserialize, Schema)]
    enum Enum {
        #[serde(rename = "a-a-a")]
        A,
        #[serde(rename = "b-b-b")]
        B,
        #[serde(rename = "c-c-c")]
        C,
    }

    #[derive(Debug, Serialize, Deserialize, Schema)]
    struct Data {
        e: Enum,
    }

    #[get("/")]
    fn index(_: Query<Data>) -> String {
        String::new()
    }

    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("a-a-a"));
    assert!(yaml.contains("b-b-b"));
    assert!(yaml.contains("c-c-c"));
}

#[test]
fn enum_variant() {
    unimplemented!()
}

#[test]
fn enum_field() {
    unimplemented!()
}

#[test]
fn enum_rename_all() {
    #[derive(Debug, Serialize, Deserialize, Schema)]
    struct Resp {
        data: String,
    }

    #[derive(Debug, Serialize, Deserialize, Schema)]
    #[serde(rename_all = "camelCase")]
    enum Enum {
        A(String),
        B { resp_data: Resp },
    }

    #[derive(Debug, Serialize, Deserialize, Schema)]
    struct Data {
        data: Enum,
    }

    #[get("/")]
    fn index(_: Query<Data>) -> String {
        String::new()
    }

    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("respData"));
}
