#![cfg(feature = "openapi")]

use std::collections::HashSet;

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, rweb::Schema)]
#[schema(component = "Things")]
struct Things {
    yarr: [u64; 24],
    yarr0: [u64; 0],
    tuple: (String, String, String),
    set: HashSet<String>,
}

#[get("/")]
fn index(_: Query<Things>) -> String {
    String::new()
}

#[test]
fn test_skip() {
    let (spec, _) = openapi::spec().build(|| index());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    let things = match schemas.get("Things").unwrap() {
        rweb::openapi::ObjectOrReference::Object(s) => s,
        _ => panic!(),
    };
    assert!(things.properties.contains_key("yarr"));
    assert!(things.properties.contains_key("yarr0"));
    assert!(things.properties.contains_key("tuple"));
    assert_eq!(things.properties.get("yarr").unwrap().min_items, Some(24));
    assert_eq!(things.properties.get("yarr").unwrap().max_items, Some(24));
    assert_eq!(
        things
            .properties
            .get("yarr")
            .unwrap()
            .items
            .as_ref()
            .unwrap()
            .schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(things.properties.get("yarr0").unwrap().min_items, Some(0));
    assert_eq!(things.properties.get("yarr0").unwrap().max_items, Some(0));
    assert_eq!(
        things
            .properties
            .get("yarr0")
            .unwrap()
            .items
            .as_ref()
            .unwrap()
            .schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(things.properties.get("tuple").unwrap().min_items, Some(3));
    assert_eq!(things.properties.get("tuple").unwrap().max_items, Some(3));
    assert_eq!(
        things
            .properties
            .get("tuple")
            .unwrap()
            .items
            .as_ref()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(
        things.properties.get("set").unwrap().unique_items,
        Some(true)
    );
    assert_eq!(
        things
            .properties
            .get("set")
            .unwrap()
            .items
            .as_ref()
            .unwrap()
            .schema_type,
        Some(openapi::Type::String)
    );
}
