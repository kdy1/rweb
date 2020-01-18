//! Guards for http methods.

use super::Guard;
use crate::http::ReqInfo;
use hyper::Method;

pub fn get() -> impl Guard {
    MethodGuard(Method::GET)
}

pub fn post() -> impl Guard {
    MethodGuard(Method::POST)
}

pub fn put() -> impl Guard {
    MethodGuard(Method::PUT)
}

pub fn connect() -> impl Guard {
    MethodGuard(Method::CONNECT)
}

pub fn delete() -> impl Guard {
    MethodGuard(Method::DELETE)
}

pub fn head() -> impl Guard {
    MethodGuard(Method::HEAD)
}

pub fn options() -> impl Guard {
    MethodGuard(Method::OPTIONS)
}

pub fn patch() -> impl Guard {
    MethodGuard(Method::PATCH)
}

pub fn trace() -> impl Guard {
    MethodGuard(Method::TRACE)
}

#[derive(Debug, Clone)]
struct MethodGuard(Method);

impl Guard for MethodGuard {
    #[inline(always)]
    fn allow(&self, req: &ReqInfo) -> bool {
        *req.method() == self.0
    }
}
