//! Automatic openapi spec generator.
//!
//!
//! # Usage
//!
//! Enable cargo feature by
//!
//!```toml
//! [dependencies]
//! rweb = { version = "0.3.0-alpha.1", features = ["openapi"] }
//! ```
//!
//! and wrap your handlers like
//!
//! ```rust
//! use rweb::*;
//! use serde::Serialize;
//!
//! #[get("/")]
//! fn index() -> String {
//!     String::from("content type will be 'text/plain' as you return String")
//! }
//!
//! #[derive(Debug, Serialize, Schema)]
//! struct Product {
//!     id: String,
//!     price: usize,
//! }
//!
//! #[get("/products")]
//! fn products() -> Json<Vec<Product>> {
//!     unimplemented!("content type will be 'application/json', and type of openapi schema will be array")
//! }
//!
//! #[get("/product/{id}")]
//! fn product(id: String) -> Json<Product> {
//!     // See Component section below if you want to give a name to type.
//!     unimplemented!("content type will be 'application/json', and type of openapi schema will be object")
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let (_spec, filter) = openapi::spec().build(||{
//!            index().or(products()).or(product())
//!     });
//!
//!     serve(filter);
//!     // Use the code below to run server.
//!     //
//!     // serve(filter).run(([127, 0, 0, 1], 3030)).await;
//! }
//! ```
//!
//! **Note**: Currently using path filter from warp is **not** supported by
//! rweb. If you use path filter from warp, generated document will point to
//! different path.
//!
//! # Annotations
//!
//! This is applicable to `#[get]`, `#[post]`, ..etc
//!
//!
//! ## `#[openapi(id = "foo")]`
//!
//! ```rust
//! use rweb::*;
//!
//! #[get("/sum/{a}/{b}")]
//! #[openapi(id = "math.sum")]
//! fn sum(a: usize, b: usize) -> String {
//!     (a + b).to_string()
//! }
//! ```
//!
//!
//! ## `#[openapi(description = "foo")]`
//!
//! ```rust
//! use rweb::*;
//!
//! /// By default, doc comments on the function will become description of the operation.
//! #[get("/sum/{a}/{b}")]
//! #[openapi(description = "But what if implementation details is written on it?")]
//! fn sum(a: usize, b: usize) -> String {
//!     (a + b).to_string()
//! }
//! ```
//!
//!
//! ## `#[openapi(summary = "foo")]`
//!
//! ```rust
//! use rweb::*;
//!
//! #[get("/sum/{a}/{b}")]
//! #[openapi(summary = "summary of operation")]
//! fn sum(a: usize, b: usize) -> String {
//!     (a + b).to_string()
//! }
//! ```
//!
//! ## `#[openapi(tags("foo", "bar"))]`
//!
//! ```rust
//! use rweb::*;
//!
//! #[get("/sum/{a}/{b}")]
//! #[openapi(tags("sum"))]
//! fn sum(a: usize, b: usize) -> String {
//!     (a + b).to_string()
//! }
//!
//! #[get("/mul/{a}/{b}")]
//! #[openapi(tags("mul"))]
//! fn mul(a: usize, b: usize) -> String {
//!     (a * b).to_string()
//! }
//!
//! // This is also applicable to #[router]
//! #[router("/math", services(sum, mul))]
//! #[openapi(tags("math"))]
//! fn math() {}
//! ```
//!
//!
//! # Parameters
//!
//! ```rust
//! use rweb::*;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, Schema)]
//! struct Opt {
//!     query: String,
//!     limit: usize,
//!     page_token: String,
//! }
//!
//! /// Look at the generated api document, and surprise :)
//! ///
//! /// Fields of [Opt] are documented as query parameters.
//! #[get("/")]
//! pub fn search(_q: Query<Opt>) -> String {
//!     String::new()
//! }
//!
//! /// Path parameter is documented. (as there's enough information to document it)
//! #[get("/{id}")]
//! pub fn get(id: String) -> String {
//!     String::new()
//! }
//!
//! /// Fields of [Opt] are documented as request body parameters.
//! pub fn store(_: Json<Opt>) -> String{
//!     String::new()
//! }
//! ```
//!
//! # Response body
//!
//! ```rust
//! use rweb::*;
//! use serde::Serialize;
//!
//! #[derive(Debug, Default, Serialize, Schema)]
//! struct Output {
//!     data: String,
//! }
//!
//! /// Json<T> implements rweb::openapi::ResponseEntity if T implements Entity.
//! #[get("/")]
//! pub fn get() -> Json<Output> {
//!     Output::default().into()
//! }
//! ```
//!
//! # `#[derive(Schema)]`
//!
//! It implements [Entity] for the struct or enum.
//!
//! ## Overriding description
//!
//! ```rust
//! use rweb::*;
//!
//! /// private documentation, for example
//! #[derive(Debug, Default, Schema)]
//! #[schema(description = "This is output!!")]
//! pub struct Output {
//!     /// By default, doc comments become description
//!     data: String,
//!     /// Another private info like implementation detail.
//!     #[schema(description = "field")]
//!     field_example: String,
//! }
//! ```
//!
//! ## Component
//!
//! ```rust
//! use rweb::*;
//! use serde::{Serialize, Deserialize};
//!
//! // This item is stored at #/components/schema/Item
//! #[derive(Debug, Serialize, Deserialize, Schema)]
//! #[schema(component = "Item")]
//! struct ComponentTestReq {
//!     data: String,
//! }
//! ```
//!
//! # Custom error
//!
//! ```rust
//! use rweb::*;
//! use std::collections::BTreeMap;
//! use std::borrow::Cow;
//!
//! #[derive(Debug, Schema)]
//! enum Error {
//!     NotFound,
//! }
//!
//! impl openapi::ResponseEntity for Error {
//!     fn describe_responses() -> openapi::Responses {
//!         let mut map = BTreeMap::new();
//!
//!         map.insert(
//!             Cow::Borrowed("404"),
//!             openapi::Response {
//!                 description: Cow::Borrowed("Item not found"),
//!                 ..Default::default()
//!             },
//!         );
//!
//!         map
//!     }
//! }
//! ```

pub use self::{
    builder::{spec, Builder},
    entity::{Components, Entity, ResponseEntity, Responses},
};
use crate::FromRequest;
use http::Method;
pub use rweb_openapi::v3_0::*;
use scoped_tls::scoped_thread_local;
use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, mem::replace};

mod builder;
mod entity;

scoped_thread_local!(static COLLECTOR: RefCell<Collector>);

#[derive(Debug)]
pub struct Collector {
    spec: Spec,
    path_prefix: String,
    tags: Vec<Cow<'static, str>>,
}

impl Collector {
    /// Method used by `#[router]`.
    #[doc(hidden)]
    pub fn with_appended_prefix<F, Ret>(
        &mut self,
        prefix: &str,
        tags: Vec<Cow<'static, str>>,
        op: F,
    ) -> Ret
    where
        F: FnOnce() -> Ret,
    {
        let orig_len = self.path_prefix.len();
        self.path_prefix.push_str(prefix);
        let orig_tag_len = self.tags.len();
        self.tags.extend(tags);

        let new = replace(self, new());
        let cell = RefCell::new(new);
        let ret = COLLECTOR.set(&cell, || op());

        let new = cell.into_inner();
        replace(self, new);

        self.tags.drain(orig_tag_len..);
        self.path_prefix.drain(orig_len..);
        ret
    }

    pub fn add_request_type_to<T: FromRequest + Entity>(&mut self, op: &mut Operation) {
        for (k, s) in T::describe_components() {
            if self.spec.components.is_none() {
                self.spec.components = Some(Default::default());
                // TODO: Error reporting
                // TODO: Remove duplicate work
                self.spec
                    .components
                    .as_mut()
                    .unwrap()
                    .schemas
                    .insert(k, ObjectOrReference::Object(s));
            }
        }

        if T::is_body() {
            if op.request_body.is_some() {
                panic!("Multiple body detected");
            }

            let s = T::describe();

            let mut content = BTreeMap::new();

            // TODO
            content.insert(
                "*/*".into(),
                MediaType {
                    schema: Some(ObjectOrReference::Object(s)),
                    examples: None,
                    encoding: Default::default(),
                },
            );

            op.request_body = Some(ObjectOrReference::Object(RequestBody {
                content,
                required: Some(!T::is_optional()),
                ..Default::default()
            }));
        }

        if T::is_query() {
            let s = T::describe();

            match s.schema_type {
                Type::Object => {
                    //
                    for (name, ty) in s.properties {
                        op.parameters.push(ObjectOrReference::Object(Parameter {
                            required: Some(s.required.contains(&name)),
                            name,
                            location: Location::Query,
                            unique_items: None,
                            description: ty.description.clone(),
                            schema: Some(ty),
                            ..Default::default()
                        }))
                    }
                }
                _ => {}
            }
        }
    }

    pub fn add_response_to<T: ResponseEntity>(&mut self, op: &mut Operation) {
        let responces = T::describe_responses();
        op.responses.extend(responces);
    }

    #[doc(hidden)]
    #[inline(never)]
    pub fn add(&mut self, path: &str, method: Method, operation: Operation) {
        let path = {
            let mut p = self.path_prefix.clone();
            p.push_str(path);
            p
        };

        let v = self
            .spec
            .paths
            .entry(Cow::Owned(path))
            .or_insert_with(Default::default);

        let op = if method == Method::GET {
            &mut v.get
        } else if method == Method::POST {
            &mut v.post
        } else if method == Method::PUT {
            &mut v.put
        } else if method == Method::DELETE {
            &mut v.delete
        } else if method == Method::HEAD {
            &mut v.head
        } else if method == Method::OPTIONS {
            &mut v.options
        } else if method == Method::CONNECT {
            unimplemented!("openapi spec generation for http CONNECT")
        } else if method == Method::PATCH {
            &mut v.patch
        } else if method == Method::TRACE {
            &mut v.trace
        } else {
            unreachable!("Unknown http method: {:?}", method)
        };

        match op {
            Some(op) => {
                assert_eq!(*op, operation);
            }
            None => {
                *op = Some(operation);
            }
        }

        let op = op.as_mut().unwrap();
        op.tags.extend(self.tags.clone());
    }

    pub fn add_scheme<T>() {}
}

fn new() -> Collector {
    Collector {
        spec: Default::default(),
        path_prefix: Default::default(),
        tags: vec![],
    }
}

#[doc(hidden)]
pub fn with<F, Ret>(op: F) -> Ret
where
    F: FnOnce(Option<&mut Collector>) -> Ret,
{
    if COLLECTOR.is_set() {
        COLLECTOR.with(|c| {
            let mut r = c.borrow_mut();
            op(Some(&mut r))
        })
    } else {
        op(None)
    }
}

/// I'm too lazy to use inflector.
#[doc(hidden)]
pub mod http_methods {
    use http::Method;

    pub const fn get() -> Method {
        Method::GET
    }

    pub const fn post() -> Method {
        Method::POST
    }

    pub const fn put() -> Method {
        Method::PUT
    }

    pub const fn delete() -> Method {
        Method::DELETE
    }

    pub const fn head() -> Method {
        Method::HEAD
    }

    pub const fn options() -> Method {
        Method::OPTIONS
    }

    pub const fn connect() -> Method {
        Method::CONNECT
    }

    pub const fn patch() -> Method {
        Method::PATCH
    }

    pub const fn trace() -> Method {
        Method::TRACE
    }
}
