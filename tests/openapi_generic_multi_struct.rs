#![cfg(feature = "openapi")]

use rweb::{openapi::*, *};
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
    _: Query<GenericStruct<Option<String>, u64>>,
    _: Json<Vec<GenericStruct<Vec<Two>, Option<GenericStruct<Option<One>, One>>>>>,
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
    assert!(schemas.contains_key("GenericStruct-string_Opt_uinteger-"));
    assert!(schemas.contains_key("One_Opt"));
    assert!(schemas.contains_key("GenericStruct-One_Opt_One-_Opt"));
    macro_rules! component {
        ($cn:expr) => {
            match schemas.get($cn) {
                Some(ObjectOrReference::Object(s)) => s,
                Some(..) => panic!("Component schema can't be a reference"),
                None => panic!("No component schema for {}", $cn),
            }
        };
    }
    assert_eq!(
        &component!("GenericStruct-One_Opt_One-_Opt").nullable,
        &Some(true)
    );
    assert_eq!(
        &component!("GenericStruct-One_Opt_One-_Opt").properties["a"],
        &ComponentOrInlineSchema::Component {
            name: rt::Cow::Borrowed("One_Opt")
        }
    );
}
