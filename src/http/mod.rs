use self::error::PayloadError;
use crate::error::Error;
use futures::Stream;
use hyper::{body::Bytes, Method, Uri, Version};
pub use hyper::{header::HeaderValue, HeaderMap};
use serde::de::DeserializeOwned;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub mod error;

/// Type represent streaming payload
pub enum Payload<S = PayloadStream> {
    None,
    // TODO: http 1 payload
    // TODO: http 2 payload
    Stream(S),
}

pub trait Response {}

type PayloadStream = Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>>>>;

pub struct Req {
    info: ReqInfo,
    body: Payload,
}

/// Request information except body.
#[derive(Debug, Clone)]
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
        &self.uri
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
        self.inner
    }
}

#[derive(Debug)]
pub struct Resp<Body>
where
    Body: MessageBody,
{
    body: Body,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Body size hint
pub enum BodySize {
    None,
    Empty,
    Sized(usize),
    Sized64(u64),
    Stream,
}

impl BodySize {
    pub fn is_eof(&self) -> bool {
        match self {
            BodySize::None | BodySize::Empty | BodySize::Sized(0) | BodySize::Sized64(0) => true,
            _ => false,
        }
    }
}

/// Type that provides this trait can be streamed to a peer.
pub trait MessageBody {
    fn size(&self) -> BodySize;

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>>;
}

impl MessageBody for () {
    fn size(&self) -> BodySize {
        BodySize::Empty
    }

    fn poll_next(&mut self, _: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        Poll::Ready(None)
    }
}

impl<T: MessageBody> MessageBody for Box<T> {
    fn size(&self) -> BodySize {
        self.as_ref().size()
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        self.as_mut().poll_next(cx)
    }
}