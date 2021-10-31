#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Schema)]
#[schema(component = "Bar")]
pub struct Bar {
    pub foo: Box<Foo>,
}

#[derive(Debug, Deserialize, Serialize, Schema)]
pub struct NotAComponent {
    pub foo: Box<Foo>,
    pub bar: Vec<Bar>,
}

#[derive(Debug, Deserialize, Serialize, Schema)]
#[schema(component = "Foo")]
pub struct Foo {
    pub interim: Option<Box<NotAComponent>>,
}

#[get("/")]
fn test_r(_: Json<Foo>) -> String {
    String::new()
}

#[test]
fn test_component_recursion_compile() {
    let (spec, _) = openapi::spec().build(|| test_r());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    for (name, _) in schemas {
        assert!(name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-'))
    }
    assert!(schemas.contains_key("Foo"));
    assert!(schemas.contains_key("Bar"));
    assert!(!schemas.contains_key("NotAComponent"));
}
