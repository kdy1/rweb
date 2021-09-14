//! https://github.com/kdy1/rweb/issues/38

#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "One")]
pub struct One {}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Two")]
pub struct Two {}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Three")]
pub struct Three {
    two: Two,
    list_of_opt_of_one: Vec<Option<One>>,
    wrapper: Wrapper,
    unit: Unit,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Wrapper")]
pub struct Wrapper(One);

#[derive(Debug, Serialize, Deserialize, Schema)]
pub struct Unit;

#[derive(Debug, Serialize, Deserialize, Schema)]
struct Relevant {
    one: One,
    three: Three,
}

#[get("/")]
fn index(_: Query<Relevant>) -> String {
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
    assert!(schemas.contains_key("One"));
    assert!(schemas.contains_key("Two"));
    assert!(schemas.contains_key("Three"));
    assert!(schemas.contains_key("One_Opt"));
    assert!(!schemas.contains_key("One_Opt_List"));
    assert!(!schemas.contains_key("One_List"));
    assert!(!schemas.contains_key("Two_List"));
    assert!(!schemas.contains_key("Three_List"));
    assert!(!schemas.contains_key("Two_Opt"));
    assert!(!schemas.contains_key("Three_Opt"));
    assert!(!schemas.contains_key("Wrapper"));
    assert!(!schemas.contains_key("Unit"));
    let schema = component!("Three");
    assert_eq!(
        schema.properties["wrapper"],
        openapi::ComponentOrInlineSchema::Component {
            name: std::borrow::Cow::Borrowed("One")
        }
    );
    assert_eq!(
        schema.properties["unit"].unwrap().unwrap().schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(
        schema.properties["unit"].unwrap().unwrap().nullable,
        Some(true)
    );
}
