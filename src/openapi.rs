//! Automatic openapi spec generator.

use crate::{Form, FromRequest, Json, Query};
use http::Method;
pub use rweb_openapi::v3_0::*;
use scoped_tls::scoped_thread_local;
use std::{cell::RefCell, collections::BTreeMap, mem::replace};

scoped_thread_local!(static COLLECTOR: RefCell<Collector>);

/// This can be derived by `#[derive(Schema)]`.
///
///
/// You may provide an example value of each field with
///
/// `#[schema(example = "path_to_function")]`
pub trait Entity {
    fn describe() -> Schema;
}

impl<T: Entity> Entity for Vec<T> {
    fn describe() -> Schema {
        Schema {
            schema_type: "array".into(),
            items: Some(Box::new(T::describe())),
            ..Default::default()
        }
    }
}

impl<T> Entity for Option<T>
where
    T: Entity,
{
    fn describe() -> Schema {
        let mut s = T::describe();
        s.nullable = Some(true);
        s
    }
}

impl<T> Entity for Json<T>
where
    T: Entity,
{
    #[inline]
    fn describe() -> Schema {
        T::describe()
    }
}

impl<T> Entity for Query<T>
where
    T: Entity,
{
    #[inline]
    fn describe() -> Schema {
        T::describe()
    }
}

impl<T> Entity for Form<T>
where
    T: Entity,
{
    #[inline]
    fn describe() -> Schema {
        T::describe()
    }
}

macro_rules! number {
    ($T:ty) => {
        impl Entity for $T {
            #[inline]
            fn describe() -> Schema {
                Schema {
                    schema_type: "number".to_string(),
                    ..Default::default()
                }
            }
        }
    };

    (
        $(
            $T:ty
        ),*
    ) => {
        $(
            number!($T);
        )*
    };
}

number!(u8, u16, u32, u64, u128, usize);
number!(i8, i16, i32, i64, i128, isize);
// TODO: non-zero types

impl Entity for String {
    fn describe() -> Schema {
        Schema {
            schema_type: "string".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Collector {
    spec: Spec,
    path_prefix: String,
    tags: Vec<String>,
}

impl Collector {
    #[doc(hidden)]
    pub fn with_appended_prefix<F, Ret>(&mut self, prefix: &str, tags: Vec<&str>, op: F) -> Ret
    where
        F: FnOnce() -> Ret,
    {
        let orig_len = self.path_prefix.len();
        self.path_prefix.push_str(prefix);
        let orig_tag_len = self.tags.len();
        self.tags.extend(tags.iter().map(|v| v.to_string()));

        let new = replace(self, new());
        let cell = RefCell::new(new);
        let ret = COLLECTOR.set(&cell, || op());

        let new = cell.into_inner();
        replace(self, new);

        self.tags.drain(orig_tag_len..);
        self.path_prefix.drain(orig_len..);
        ret
    }

    pub fn add_request_type_to<T: FromRequest + Entity>(mut op: Operation) -> Operation {
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

            match &*s.schema_type {
                "object" => {
                    //
                    for (name, ty) in s.properties {
                        op.parameters.push(ObjectOrReference::Object(Parameter {
                            required: Some(s.required.contains(&name)),
                            name,
                            location: "query".to_string(),
                            unique_items: None,
                            description: ty.description.clone(),
                            schema: Some(ty),
                            ..Default::default()
                        }))
                    }
                }
                _ => unimplemented!("other type than object"),
            }
        }

        op
    }

    #[doc(hidden)]
    #[inline(never)]
    pub fn add(&mut self, path: &str, method: Method, operation: Operation) {
        let path = {
            let mut p = self.path_prefix.clone();
            p.push_str(path);
            p
        };

        let v = self.spec.paths.entry(path).or_insert_with(Default::default);

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

pub fn spec<F, Ret>(op: F) -> (Spec, Ret)
where
    F: FnOnce() -> Ret,
{
    let collector = new();

    let cell = RefCell::new(collector);

    let ret = COLLECTOR.set(&cell, || op());
    let mut spec = cell.into_inner().spec;
    spec.openapi = "3.0.1".into();
    (spec, ret)
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
