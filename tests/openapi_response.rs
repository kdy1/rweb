#![cfg(feature = "openapi")]

use rweb::*;
use serde::Serialize;

#[derive(Debug, Serialize, Schema)]
struct Resp<T> {
    status: usize,
    data: T,
}

#[get("/")]
fn index() -> Resp<()> {
    unimplemented!()
}

#[get("/example")]
#[response()]
fn example() -> Resp<()> {
    unimplemented!()
}

#[test]
fn component_test() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    panic!()
}
