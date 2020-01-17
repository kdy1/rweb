use hyper::{HeaderMap, Method, Uri, Version};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// Request information except body.
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

#[derive(Debug)]
pub enum RequestError {}

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
