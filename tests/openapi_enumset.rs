#![cfg(feature = "openapi")]
#![cfg(feature = "enumset")]

use enumset::*;
use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(EnumSetType, Schema, Serialize, Deserialize, Debug)]
#[schema(component = "Flagged")]
pub enum Flagged {
    A,
    B,
    C,
}

#[derive(EnumSetType, Schema, Serialize, Deserialize, Debug)]
#[enumset(serialize_as_list)]
#[schema(component = "Named")]
pub enum Named {
    A,
    B,
    C,
}

#[get("/")]
fn index(_: Query<EnumSet<Flagged>>, _: Json<EnumSet<Named>>) -> String {
    String::new()
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    macro_rules! component {
        ($cn:expr) => {
            match schemas.get($cn) {
                Some(openapi::ObjectOrReference::Object(s)) => s,
                Some(..) => panic!("Component schema can't be a reference"),
                None => panic!("No component schema for {}", $cn),
            }
        };
    }
    let flagged = component!("Flagged_EnumSet");
    assert_eq!(flagged.schema_type, Some(openapi::Type::Integer));
    let named = component!("Named_EnumSet");
    assert_eq!(named.schema_type, Some(openapi::Type::Array));
    assert!(named.items.is_some());
}
