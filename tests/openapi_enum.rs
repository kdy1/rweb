#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "EExttagged")]
enum EExttagged {
    A(String),
    B(usize),
    Stru { field: String },
    Plain,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[serde(tag = "tag")]
#[schema(component = "EInttagged")]
enum EInttagged {
    Stru { field: String },
    Plain,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[serde(tag = "tag", content = "content")]
#[schema(component = "EAdjtagged")]
enum EAdjtagged {
    A(String),
    B(usize),
    Stru { field: String },
    Plain,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[serde(untagged)]
#[schema(component = "EUntagged")]
enum EUntagged {
    A(String),
    B(usize),
    Stru { field: String },
}

#[derive(Debug, Serialize, Deserialize, Schema)]
struct Enums {
    ext: EExttagged,
    adj: EAdjtagged,
    int: EInttagged,
    unt: EUntagged,
}

#[get("/")]
fn index(_: Json<Enums>) -> String {
    String::new()
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    println!("{}", serde_yaml::to_string(&spec).unwrap());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    macro_rules! component {
        ($cn:expr) => {
            match schemas.get($cn) {
                Some(openapi::ObjectOrReference::Object(s)) => s,
                Some(..) => panic!("Component schema can't be a reference"),
                None => panic!("No component schema for {}", $cn),
            }
        };
    }
    println!("{}", serde_yaml::to_string(&EExttagged::B(55)).unwrap());
    println!("{}", serde_yaml::to_string(&EExttagged::Plain).unwrap());
    let schema = component!("EExttagged");
    assert_eq!(schema.one_of.len(), 4);
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(schema.one_of[0].unwrap().unwrap().required, vec!["A"]);
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().properties["A"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(schema.one_of[1].unwrap().unwrap().required, vec!["B"]);
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().properties["B"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(
        schema.one_of[2].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(schema.one_of[2].unwrap().unwrap().required, vec!["Stru"]);
    assert_eq!(
        schema.one_of[2].unwrap().unwrap().properties["Stru"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(
        schema.one_of[3].unwrap().unwrap().schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[3].unwrap().unwrap().enum_values,
        vec!["Plain"]
    );
    let schema = component!("EInttagged");
    assert_eq!(schema.one_of.len(), 2);
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().required,
        vec!["field", "tag"]
    );
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().properties["field"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .enum_values,
        vec!["Stru"]
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(schema.one_of[1].unwrap().unwrap().required, vec!["tag"]);
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .enum_values,
        vec!["Plain"]
    );
    let schema = component!("EAdjtagged");
    assert_eq!(schema.one_of.len(), 4);
    for s in &schema.one_of {
        assert_eq!(s.unwrap().unwrap().schema_type, Some(openapi::Type::Object));
        assert_eq!(
            s.unwrap().unwrap().properties["tag"]
                .unwrap()
                .unwrap()
                .schema_type,
            Some(openapi::Type::String)
        );
    }
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().required,
        vec!["tag", "content"]
    );
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .enum_values,
        vec!["A"]
    );
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().properties["content"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().required,
        vec!["tag", "content"]
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .enum_values,
        vec!["B"]
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().properties["content"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(
        schema.one_of[2].unwrap().unwrap().required,
        vec!["tag", "content"]
    );
    assert_eq!(
        schema.one_of[2].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .enum_values,
        vec!["Stru"]
    );
    assert_eq!(
        schema.one_of[2].unwrap().unwrap().properties["content"]
            .unwrap()
            .unwrap()
            .schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(schema.one_of[3].unwrap().unwrap().required, vec!["tag"]);
    assert_eq!(
        schema.one_of[3].unwrap().unwrap().properties["tag"]
            .unwrap()
            .unwrap()
            .enum_values,
        vec!["Plain"]
    );
    let schema = component!("EUntagged");
    assert_eq!(schema.one_of.len(), 3);
    assert_eq!(
        schema.one_of[0].unwrap().unwrap().schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        schema.one_of[1].unwrap().unwrap().schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(
        schema.one_of[2].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
}
