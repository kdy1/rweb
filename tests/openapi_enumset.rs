#![cfg(feature = "openapi")]
#![cfg(feature = "enumset")]

use enumset::*;
use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(EnumSetType, Schema, Serialize, Deserialize, Debug)]
pub enum Flagged {
    A,
    B,
    C,
}

#[derive(EnumSetType, Schema, Serialize, Deserialize, Debug)]
#[enumset(serialize_as_list)]
pub enum Named {
    A,
    B,
    C,
}

#[derive(Schema, Serialize, Deserialize, Debug)]
#[schema(component = "Components")]
struct Components {
    flagged: EnumSet<Flagged>,
    named: EnumSet<Named>,
}

#[get("/")]
fn index(_: Json<Components>) -> String {
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
    let components = component!("Components");
    macro_rules! unpack {
        ($opt:expr) => {
            $opt.unwrap().unwrap().unwrap()
        };
    }
    macro_rules! prop {
        ($prop:expr) => {
            unpack!(components.properties.get($prop))
        };
    }
    let flagged = prop!("flagged");
    assert_eq!(flagged.schema_type, Some(openapi::Type::Integer));
    let named = prop!("named");
    assert_eq!(named.schema_type, Some(openapi::Type::Array));
    assert!(named.items.is_some());
}
