//! Automatic openapi spec generator.

pub use self::{
    builder::{spec, Builder},
    entity::Entity,
};
use crate::{openapi::entity::ResponseEntity, FromRequest};
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
        let comp = T::describe_component();

        if let Some((k, s)) = comp.clone() {
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
            let s = comp.map(|v| v.1).unwrap_or_else(T::describe);

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
        let resp = T::describe_response();
        op.responses.insert("200".into(), resp);
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
