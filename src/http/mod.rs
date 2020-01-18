use self::error::PayloadError;
use crate::error::Error;
use bytes::BytesMut;
use futures::Stream;
use http::response::Parts;
use hyper::{body::Bytes, Method, Uri, Version};
pub use hyper::{header::HeaderValue, http::StatusCode, HeaderMap};
use pin_project::pin_project;
use serde::de::DeserializeOwned;
use std::{
    marker::PhantomData,
    mem::replace,
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

type PayloadStream = Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>>>>;

pub struct Req {
    info: ReqInfo,
    body: Payload,
}

impl Req {
    pub fn head(&self) -> &ReqInfo {
        &self.info
    }
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
pub struct Resp<B = Body>
where
    B: MessageBody,
{
    head: Parts,
    body: Option<B>,
}

impl<B> Resp<B>
where
    B: MessageBody,
{
    pub fn build(status: StatusCode) -> RespBuilder<B> {
        RespBuilder { status, body: None }
    }

    pub fn headers(&mut self) -> &HeaderMap {
        &self.head.headers
    }

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.head.headers
    }
}

#[derive(Debug, Clone)]
pub struct RespBuilder<B>
where
    B: MessageBody,
{
    status: StatusCode,
    body: Option<B>,
}

impl<B> RespBuilder<B>
where
    B: MessageBody,
{
    pub fn body(mut self, body: B) -> Self {
        self.body = Some(body);
        self
    }
}

pub enum Body {
    None,
    Empty,
    Bytes(Bytes),
    Message(Box<dyn MessageBody>),
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

impl Body {
    /// Create body from slice (copy)
    pub fn from_slice(s: &[u8]) -> Body {
        Body::Bytes(Bytes::copy_from_slice(s))
    }

    /// Create body from generic message body.
    pub fn from_message<B: MessageBody + 'static>(body: B) -> Body {
        Body::Message(Box::new(body))
    }
}

impl MessageBody for Body {
    fn size(&self) -> BodySize {
        match self {
            Body::None => BodySize::None,
            Body::Empty => BodySize::Empty,
            Body::Bytes(ref bin) => BodySize::Sized(bin.len()),
            Body::Message(ref body) => body.size(),
        }
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        match self {
            Body::None => Poll::Ready(None),
            Body::Empty => Poll::Ready(None),
            Body::Bytes(ref mut bin) => {
                let len = bin.len();
                if len == 0 {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(replace(bin, Bytes::new()))))
                }
            }
            Body::Message(ref mut body) => body.poll_next(cx),
        }
    }
}

impl From<&'static str> for Body {
    fn from(s: &'static str) -> Body {
        Body::Bytes(Bytes::from_static(s.as_ref()))
    }
}

impl From<&'static [u8]> for Body {
    fn from(s: &'static [u8]) -> Body {
        Body::Bytes(Bytes::from_static(s))
    }
}

impl From<Vec<u8>> for Body {
    fn from(vec: Vec<u8>) -> Body {
        Body::Bytes(Bytes::from(vec))
    }
}

impl From<String> for Body {
    fn from(s: String) -> Body {
        s.into_bytes().into()
    }
}

impl<'a> From<&'a String> for Body {
    fn from(s: &'a String) -> Body {
        Body::Bytes(Bytes::copy_from_slice(AsRef::<[u8]>::as_ref(&s)))
    }
}

impl From<Bytes> for Body {
    fn from(s: Bytes) -> Body {
        Body::Bytes(s)
    }
}

impl From<BytesMut> for Body {
    fn from(s: BytesMut) -> Body {
        Body::Bytes(s.freeze())
    }
}

impl From<serde_json::Value> for Body {
    fn from(v: serde_json::Value) -> Body {
        Body::Bytes(v.to_string().into())
    }
}

impl<S> From<SizedStream<S>> for Body
where
    S: Stream<Item = Result<Bytes, Error>> + 'static,
{
    fn from(s: SizedStream<S>) -> Body {
        Body::from_message(s)
    }
}

impl<S, E> From<BodyStream<S, E>> for Body
where
    S: Stream<Item = Result<Bytes, E>> + 'static,
    E: Into<Error> + 'static,
{
    fn from(s: BodyStream<S, E>) -> Body {
        Body::from_message(s)
    }
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

/// Type represent streaming body.
/// Response does not contain `content-length` header and appropriate transfer
/// encoding is used.
#[pin_project]
pub struct BodyStream<S, E> {
    #[pin]
    stream: S,
    _t: PhantomData<E>,
}

impl<S, E> BodyStream<S, E>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<Error>,
{
    pub fn new(stream: S) -> Self {
        BodyStream {
            stream,
            _t: PhantomData,
        }
    }
}

impl<S, E> MessageBody for BodyStream<S, E>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<Error>,
{
    fn size(&self) -> BodySize {
        BodySize::Stream
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        unsafe { Pin::new_unchecked(self) }
            .project()
            .stream
            .poll_next(cx)
            .map(|res| res.map(|res| res.map_err(std::convert::Into::into)))
    }
}

/// Type represent streaming body. This body implementation should be used
/// if total size of stream is known. Data get sent as is without using transfer
/// encoding.
#[pin_project]
pub struct SizedStream<S> {
    size: u64,
    #[pin]
    stream: S,
}

impl<S> SizedStream<S>
where
    S: Stream<Item = Result<Bytes, Error>>,
{
    pub fn new(size: u64, stream: S) -> Self {
        SizedStream { size, stream }
    }
}

impl<S> MessageBody for SizedStream<S>
where
    S: Stream<Item = Result<Bytes, Error>>,
{
    fn size(&self) -> BodySize {
        BodySize::Sized64(self.size)
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        unsafe { Pin::new_unchecked(self) }
            .project()
            .stream
            .poll_next(cx)
    }
}
