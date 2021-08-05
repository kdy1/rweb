#![cfg(feature = "openapi")]
#![allow(dead_code)]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, rweb::Schema)]
#[schema(component = "Whence")]
struct Whence {
    always: u64,
    #[serde(skip_deserializing)]
    only_yeet: u64,
    #[serde(skip_serializing)]
    only_take: u64,
    #[serde(skip)]
    nevah: u64,
}

#[get("/")]
fn index(_: Query<Whence>) -> String {
    String::new()
}

#[test]
fn test_skip() {
    let (spec, _) = openapi::spec().build(|| index());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    let whence = match schemas.get("Whence").unwrap() {
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
            unpack!(whence.properties.get($prop))
        };
    }
    assert!(whence.properties.contains_key("always"));
    assert!(whence.properties.contains_key("only_yeet"));
    assert!(whence.properties.contains_key("only_take"));
    assert!(!whence.properties.contains_key("nevah"));
    assert_eq!(prop!("always").read_only, None);
    assert_eq!(prop!("always").write_only, None);
    assert_eq!(prop!("only_yeet").read_only, Some(true));
    assert_eq!(prop!("only_yeet").write_only, None);
    assert_eq!(prop!("only_take").read_only, None);
    assert_eq!(prop!("only_take").write_only, Some(true));
}
