#![cfg(feature = "openapi")]

use rweb::openapi::*;
use rweb::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "One")]
pub struct One {}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Two")]
pub struct Two {}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "GenericStruct")]
struct GenericStruct<A, B> {
    a: A,
    b: HashMap<String, B>,
}

#[get("/")]
fn test_r(
    _: Query<GenericStruct<Option<One>, u64>>,
    _: Json<Vec<GenericStruct<Vec<Two>, Option<GenericStruct<One, One>>>>>,
) -> String {
    String::new()
}

#[test]
fn test_multi_generics_compile() {
    let (spec, _) = openapi::spec().build(|| test_r());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    for (name, _) in schemas {
        assert!(name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-'))
    }
    assert!(schemas.contains_key("One"));
    assert!(schemas.contains_key("Two"));
    assert!(schemas.contains_key("One_Opt"));
    assert!(schemas.contains_key("One_Map"));
    assert!(schemas.contains_key("Two_List"));
    assert!(schemas.contains_key("GenericStruct-_One_One_-_Opt"));
    macro_rules! component {
        ($cn:expr) => {
            match schemas.get($cn) {
                Some(ObjectOrReference::Object(s)) => s,
                Some(..) => panic!("Component schema can't be a reference"),
                None => panic!("No component schema for {}", $cn),
            }
        };
    }
    match &component!("One_Opt").one_of[0] {
        ObjectOrReference::Object(s) => {
            assert_eq!(s.ref_path, "#/components/schemas/One");
            assert_eq!(s.schema_type, None);
        }
        ObjectOrReference::Ref { ref_path } => assert_eq!(ref_path, "#/components/schemas/One"),
    }
    match &component!("One_Map").additional_properties {
        Some(ObjectOrReference::Object(s)) => {
            assert_eq!(s.ref_path, "#/components/schemas/One");
            assert_eq!(s.schema_type, None);
        }
        Some(ObjectOrReference::Ref { ref_path }) => {
            assert_eq!(ref_path, "#/components/schemas/One")
        }
        None => panic!("Map component missing `additional_properties`"),
    }
    match &component!("Two_List").items {
        Some(s) => {
            assert_eq!(s.ref_path, "#/components/schemas/Two");
            assert_eq!(s.schema_type, None);
        }
        None => panic!("Array component missing `items`"),
    }
}
