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
    macro_rules! unpack {
        ($opt:expr) => {
            $opt.unwrap().unwrap().unwrap()
        };
    }
    macro_rules! prop {
        ($prop:expr) => {
            unpack!(things.properties.get($prop))
        };
    }
    assert!(things.properties.contains_key("yarr"));
    assert!(things.properties.contains_key("yarr0"));
    assert!(things.properties.contains_key("tuple"));
    assert_eq!(prop!("yarr").min_items, Some(24));
    assert_eq!(prop!("yarr").max_items, Some(24));
    assert_eq!(
        unpack!(prop!("yarr").items.as_ref()).schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(prop!("yarr0").min_items, Some(0));
    assert_eq!(prop!("yarr0").max_items, Some(0));
    assert_eq!(
        unpack!(prop!("yarr0").items.as_ref()).schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(prop!("tuple").min_items, Some(3));
    assert_eq!(prop!("tuple").max_items, Some(3));
    assert_eq!(
        unpack!(prop!("tuple").items.as_ref()).schema_type,
        Some(openapi::Type::String)
    );
    assert_eq!(prop!("set").unique_items, Some(true));
    assert_eq!(
        unpack!(prop!("set").items.as_ref()).schema_type,
        Some(openapi::Type::String)
    );
}
