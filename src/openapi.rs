//! Automatic openapi spec generator.

use http::Method;
pub use rweb_openapi::v3_0::*;
use scoped_tls::scoped_thread_local;
use std::cell::RefCell;

scoped_thread_local!(static COLLECTOR: RefCell<Collector>);

#[derive(Debug)]
pub struct Collector {
    spec: Spec,
    path_prefix: String,
}

impl Collector {
    pub fn with_appended_prefix<F>(&mut self, prefix: &str, op: F)
    where
        F: FnOnce(&mut Self),
    {
        let orig_len = self.path_prefix.len();
        self.path_prefix.push_str(prefix);
        op(self);
        self.path_prefix.drain(orig_len..);
    }

    /// Do not call this by hand.
    #[inline(never)]
    pub fn add(&mut self, path: String, method: Method, operation: Operation) {
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
    }
}

pub fn spec<F, Ret>(op: F) -> (Spec, Ret)
where
    F: FnOnce() -> Ret,
{
    let collector = Collector {
        spec: Default::default(),
        path_prefix: Default::default(),
    };

    let cell = RefCell::new(collector);

    let ret = COLLECTOR.set(&cell, || op());

    (cell.into_inner().spec, ret)
}

pub fn with<F>(op: F)
where
    F: FnOnce(&mut Collector),
{
    if COLLECTOR.is_set() {
        COLLECTOR.with(|c| {
            let mut r = c.borrow_mut();
            op(&mut r);
        });
    }
}
