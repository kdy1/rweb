use self::error::PayloadError;
use crate::{error::Error, HttpMessage};
use bytes::BytesMut;
use futures::Stream;
use http::{header::Entry, response::Parts, Extensions};
use hyper::{body::Bytes, Method, Request, Uri, Version};
pub use hyper::{header::HeaderValue, http::StatusCode, HeaderMap};
use pin_project::pin_project;
use serde::de::DeserializeOwned;
use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    mem::replace,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

pub mod error;
pub mod msg;

/// Type represent boxed payload
pub type PayloadStream = Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>>>>;

/// Type represent streaming payload
pub enum Payload<S = PayloadStream> {
    None,
    Stream(S),
}

impl From<PayloadStream> for Payload {
    fn from(pl: PayloadStream) -> Self {
        Payload::Stream(pl)
    }
}

impl<S> Payload<S> {
    /// Takes current payload and replaces it with `None` value
    pub fn take(&mut self) -> Payload<S> {
        std::mem::replace(self, Payload::None)
    }
}

impl<S> Stream for Payload<S>
where
    S: Stream<Item = Result<Bytes, PayloadError>> + Unpin,
{
    type Item = Result<Bytes, PayloadError>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Payload::None => Poll::Ready(None),
            Payload::Stream(ref mut pl) => Pin::new(pl).poll_next(cx),
        }
    }
}

#[derive(Clone)]
pub struct Req {
    inner: Rc<Request<Payload>>,
}

impl Req {
    pub fn app_data<T>(&self) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.inner.extensions().get()
    }

    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }

    pub fn query_string(&self) -> Option<&str> {
        self.inner.uri().query()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers()
    }

    /// Create service response for error
    #[inline]
    pub fn error_response<E: Into<Error>>(self, err: E) -> Resp {
        Resp::from_err(err, self)
    }
}

impl HttpMessage for Req {
    type Stream = PayloadStream;

    #[inline]
    /// Returns Request's headers.
    fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    #[inline]
    fn take_payload(&mut self) -> Payload<Self::Stream> {
        self.inner.body_mut().take()
    }

    /// Request extensions
    #[inline]
    fn extensions(&self) -> Ref<'_, Extensions> {
        self.inner.extensions()
    }

    /// Mutable reference to a the request's extensions
    #[inline]
    fn extensions_mut(&self) -> RefMut<'_, Extensions> {
        self.inner.extensions_mut()
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

macro_rules! static_resp {
    ($name:ident, $status:expr) => {
        pub fn $name() -> RespBuilder<Body> {
            Resp::builder($status)
        }
    };
}

impl Resp {
    static_resp!(ok, StatusCode::OK);
    static_resp!(created, StatusCode::CREATED);
    static_resp!(accepted, StatusCode::ACCEPTED);
    static_resp!(
        non_authoritative_information,
        StatusCode::NON_AUTHORITATIVE_INFORMATION
    );

    static_resp!(no_content, StatusCode::NO_CONTENT);
    static_resp!(reset_content, StatusCode::RESET_CONTENT);
    static_resp!(partial_content, StatusCode::PARTIAL_CONTENT);
    static_resp!(multi_status, StatusCode::MULTI_STATUS);
    static_resp!(already_reported, StatusCode::ALREADY_REPORTED);

    static_resp!(multiple_choices, StatusCode::MULTIPLE_CHOICES);
    static_resp!(moved_permanently, StatusCode::MOVED_PERMANENTLY);
    static_resp!(found, StatusCode::FOUND);
    static_resp!(see_other, StatusCode::SEE_OTHER);
    static_resp!(not_modified, StatusCode::NOT_MODIFIED);
    static_resp!(use_proxy, StatusCode::USE_PROXY);
    static_resp!(temporary_redirect, StatusCode::TEMPORARY_REDIRECT);
    static_resp!(permanent_redirect, StatusCode::PERMANENT_REDIRECT);

    static_resp!(bad_request, StatusCode::BAD_REQUEST);
    static_resp!(not_found, StatusCode::NOT_FOUND);
    static_resp!(unauthorized, StatusCode::UNAUTHORIZED);
    static_resp!(payment_required, StatusCode::PAYMENT_REQUIRED);
    static_resp!(forbidden, StatusCode::FORBIDDEN);
    static_resp!(method_not_allowed, StatusCode::METHOD_NOT_ALLOWED);
    static_resp!(not_acceptable, StatusCode::NOT_ACCEPTABLE);
    static_resp!(
        proxy_authentication_required,
        StatusCode::PROXY_AUTHENTICATION_REQUIRED
    );
    static_resp!(request_timeout, StatusCode::REQUEST_TIMEOUT);
    static_resp!(conflict, StatusCode::CONFLICT);
    static_resp!(gone, StatusCode::GONE);
    static_resp!(length_required, StatusCode::LENGTH_REQUIRED);
    static_resp!(precondition_failed, StatusCode::PRECONDITION_FAILED);
    static_resp!(precondition_required, StatusCode::PRECONDITION_REQUIRED);
    static_resp!(payload_too_large, StatusCode::PAYLOAD_TOO_LARGE);
    static_resp!(uri_too_long, StatusCode::URI_TOO_LONG);
    static_resp!(unsupported_media_type, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    static_resp!(range_not_satisfiable, StatusCode::RANGE_NOT_SATISFIABLE);
    static_resp!(expectation_failed, StatusCode::EXPECTATION_FAILED);
    static_resp!(unprocessable_entity, StatusCode::UNPROCESSABLE_ENTITY);
    static_resp!(too_many_requests, StatusCode::TOO_MANY_REQUESTS);

    static_resp!(internal_server_error, StatusCode::INTERNAL_SERVER_ERROR);
    static_resp!(not_implemented, StatusCode::NOT_IMPLEMENTED);
    static_resp!(bad_gateway, StatusCode::BAD_GATEWAY);
    static_resp!(service_unavailable, StatusCode::SERVICE_UNAVAILABLE);
    static_resp!(gateway_timeout, StatusCode::GATEWAY_TIMEOUT);
    static_resp!(
        version_not_supported,
        StatusCode::HTTP_VERSION_NOT_SUPPORTED
    );
    static_resp!(variant_also_negotiates, StatusCode::VARIANT_ALSO_NEGOTIATES);
    static_resp!(insufficient_storage, StatusCode::INSUFFICIENT_STORAGE);
    static_resp!(loop_detected, StatusCode::LOOP_DETECTED);
}

impl Resp {
    /// Create service response from the error
    pub fn from_err<E: Into<Error>>(err: E, _request: Req) -> Self {
        let e: Error = err.into();
        let res: Resp = e.as_response_error().error_response();
        res
    }
}

impl<B> Resp<B>
where
    B: MessageBody,
{
    pub fn new(status: StatusCode) -> Resp<B> {
        let (head, _) = http::response::Builder::new()
            .status(status)
            .body::<Option<B>>(None)
            .expect("This cannot fail")
            .into_parts();
        Resp { head, body: None }
    }

    pub fn builder(status: StatusCode) -> RespBuilder<B> {
        let (head, _) = http::response::Builder::new()
            .status(status)
            .body::<Option<B>>(None)
            .expect("This cannot fail")
            .into_parts();

        RespBuilder { head, body: None }
    }

    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    pub fn headers(&mut self) -> &HeaderMap {
        &self.head.headers
    }

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.head.headers
    }

    pub fn set_body(&mut self, body: B) {
        self.body = Some(body);
    }
}

#[derive(Debug)]
pub struct RespBuilder<B>
where
    B: MessageBody,
{
    head: Parts,
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

    pub fn content_type(mut self, s: &str) -> Self {
        match self.head.headers.entry("Content-Type") {
            Entry::Occupied(mut e) => {
                e.insert(HeaderValue::from_str(s).expect("invalid content type"));
            }
            Entry::Vacant(e) => {
                e.insert(HeaderValue::from_str(s).expect("invalid content type"));
            }
        };

        self
    }

    pub fn build(self) -> Resp<B> {
        self.into()
    }
}

impl<B> From<RespBuilder<B>> for Resp<B>
where
    B: MessageBody,
{
    fn from(b: RespBuilder<B>) -> Self {
        Resp {
            head: b.head,
            body: b.body,
        }
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
