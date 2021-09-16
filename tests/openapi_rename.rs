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
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        data_msg: String,
    }
    #[derive(Debug, Serialize, Deserialize, Schema)]
    #[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
    struct Data2 {
        data2_msg: String,
    }

    #[get("/")]
    fn index(_: Query<Data>) -> Json<Data2> {
        Json::from(Data2 {
            data2_msg: String::new(),
        })
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
    assert!(yaml.contains("DATA2_MSG"));
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
fn enum_field() {
    #[derive(Debug, Serialize, Deserialize, Schema)]
    enum Enum {
        Msg {
            #[serde(rename = "msg")]
            message: String,
        },
    }

    #[derive(Debug, Serialize, Deserialize, Schema)]
    struct Data {
        #[serde(rename = "result")]
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

    assert!(yaml.contains("msg"));
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

#[test]
fn enum_rename_all_variant() {
    #[derive(Debug, Serialize, Deserialize, Schema)]
    #[serde(rename_all = "camelCase")]
    enum Enum {
        Foo,
        Bar,
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

    assert!(yaml.contains("foo"));
    assert!(yaml.contains("bar"));
}
