use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Method, Request, Response, Server, Uri, Version};
use serde::de::DeserializeOwned;
use serde::export::PhantomData;
use std::borrow::Cow;
use std::net::SocketAddr;

/// Request information except body.
///
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ReqInfo {
    method: Method,
    uri: Uri,
    version: Version,
    /// The request's headers.
    pub headers: HeaderMap,
}

impl ReqInfo {
    /// The request's method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The request's URI
    pub fn uri(&self) -> &Uri {
        &self.url
    }

    /// The request's version
    pub fn version(&self) -> Version {
        self.version
    }
}

/// Typically generated with macros.
pub trait ServiceFactory {}

/// A request from client.
#[derive(Debug)]
pub struct Data<T>
where
    T: DeserializeOwned,
{
    inner: Result<T, RequestError>,
    _phantom: PhantomData<T>,
}

impl<T> Data<T>
where
    T: DeserializeOwned,
{
    pub fn take(self) -> Result<T, RequestError> {
        self.data
    }
}

/// An application builder.
#[derive(Default)]
pub struct App {
    services: Vec<Box<dyn 'static + ServiceFactory>>,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn service(mut self, svc: impl 'static + ServiceFactory) -> Self {
        self.services.push(Box::new(svc));
        self
    }
}
