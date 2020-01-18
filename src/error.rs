pub use crate::http::error::PayloadError;
use crate::{
    http::{Body, StatusCode},
    Resp,
};
use bytes::BytesMut;
use derive_more::{Display, From};
use http::uri::InvalidUri;
use hyper::{header, http};
use serde_json::error::Error as JsonError;
use serde_urlencoded::ser::Error as FormError;
use std::{
    any::TypeId,
    cell::RefCell,
    convert::From,
    fmt,
    fmt::{Debug, Display},
    io,
    str::Utf8Error,
    string::FromUtf8Error,
};

/// Errors which can occur when attempting to generate resource uri.
#[derive(Debug, Display, From)]
pub enum UrlGenerationError {
    /// Resource not found
    #[display(fmt = "Resource not found")]
    ResourceNotFound,
    /// Not all path pattern covered
    #[display(fmt = "Not all path pattern covered")]
    NotEnoughElements,
    /// URL parse error
    #[display(fmt = "{}", _0)]
    ParseError(http::uri::InvalidUri),
}

/// `InternalServerError` for `UrlGeneratorError`
impl ResponseError for UrlGenerationError {}

/// General purpose actix web error.
///
/// An actix web error is used to carry errors from `failure` or `std::error`
/// through actix in a convenient way.  It can be created through
/// converting errors with `into()`.
///
/// Whenever it is created from an external object a response error is created
/// for it that can be used to create an http response from it this means that
/// if you have access to an actix `Error` you can always get a
/// `ResponseError` reference from it.
#[derive(From)]
pub struct Error {
    cause: Box<dyn ResponseError>,
}

impl<T> From<T> for Error
where
    T: ResponseError + 'static,
{
    fn from(e: T) -> Self {
        Error { cause: Box::new(e) }
    }
}

impl Error {
    /// Returns the reference to the underlying `ResponseError`.
    pub fn as_response_error(&self) -> &dyn ResponseError {
        self.cause.as_ref()
    }

    /// Similar to `as_response_error` but downcasts.
    pub fn as_error<T: ResponseError + 'static>(&self) -> Option<&T> {
        ResponseError::downcast_ref(self.cause.as_ref())
    }
}

/// Error that can be converted to `Response`
pub trait ResponseError: Debug + Display {
    /// Response's status code
    ///
    /// Internal server error is generated by default.
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Create response for error
    ///
    /// Internal server error is generated by default.
    fn error_response(&self) -> Resp {
        use std::fmt::Write;

        let mut resp = Resp::new(self.status_code());
        let mut buf = BytesMut::new();
        let _ = write!(&mut buf, "{}", self);
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        resp.set_body(Body::from(buf));
        resp
    }

    #[doc(hidden)]
    fn __private_get_type_id__(&self) -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }
}

impl dyn ResponseError + 'static {
    /// Downcasts a response error to a specific type.
    pub fn downcast_ref<T: ResponseError + 'static>(&self) -> Option<&T> {
        if self.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const dyn ResponseError as *const T)) }
        } else {
            None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.cause)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

/// A set of errors that can occur during parsing urlencoded payloads
#[derive(Debug, Display, From)]
pub enum UrlencodedError {
    /// Can not decode chunked transfer encoding
    #[display(fmt = "Can not decode chunked transfer encoding")]
    Chunked,
    /// Payload size is bigger than allowed. (default: 256kB)
    #[display(
        fmt = "Urlencoded payload size is bigger ({} bytes) than allowed (default: {} bytes)",
        size,
        limit
    )]
    Overflow { size: usize, limit: usize },
    /// Payload size is now known
    #[display(fmt = "Payload size is now known")]
    UnknownLength,
    /// Content type error
    #[display(fmt = "Content type error")]
    ContentType,
    /// Parse error
    #[display(fmt = "Parse error")]
    Parse,
    /// Payload error
    #[display(fmt = "Error that occur during reading payload: {}", _0)]
    Payload(PayloadError),
}

/// Return `BadRequest` for `UrlencodedError`
impl ResponseError for UrlencodedError {
    fn status_code(&self) -> StatusCode {
        match *self {
            UrlencodedError::Overflow { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            UrlencodedError::UnknownLength => StatusCode::LENGTH_REQUIRED,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

/// A set of errors that can occur during parsing json payloads
#[derive(Debug, Display, From)]
pub enum JsonPayloadError {
    /// Payload size is bigger than allowed. (default: 32kB)
    #[display(fmt = "Json payload size is bigger than allowed")]
    Overflow,
    /// Content type error
    #[display(fmt = "Content type error")]
    ContentType,
    /// Deserialize error
    #[display(fmt = "Json deserialize error: {}", _0)]
    Deserialize(JsonError),
    /// Payload error
    #[display(fmt = "Error that occur during reading payload: {}", _0)]
    Payload(PayloadError),
}

/// Return `BadRequest` for `JsonPayloadError`
impl ResponseError for JsonPayloadError {
    fn error_response(&self) -> Resp {
        match *self {
            JsonPayloadError::Overflow => Resp::new(StatusCode::PAYLOAD_TOO_LARGE),
            _ => Resp::new(StatusCode::BAD_REQUEST),
        }
    }
}

/// A set of errors that can occur during parsing request paths
#[derive(Debug, Display, From)]
pub enum PathError {
    /// Deserialize error
    #[display(fmt = "Path deserialize error: {}", _0)]
    Deserialize(serde::de::value::Error),
}

/// Return `BadRequest` for `PathError`
impl ResponseError for PathError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// A set of errors that can occur during parsing query strings
#[derive(Debug, Display, From)]
pub enum QueryPayloadError {
    /// Deserialize error
    #[display(fmt = "Query deserialize error: {}", _0)]
    Deserialize(serde::de::value::Error),
}

/// Return `BadRequest` for `QueryPayloadError`
impl ResponseError for QueryPayloadError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// Error type returned when reading body as lines.
#[derive(From, Display, Debug)]
pub enum ReadlinesError {
    /// Error when decoding a line.
    #[display(fmt = "Encoding error")]
    /// Payload size is bigger than allowed. (default: 256kB)
    EncodingError,
    /// Payload error.
    #[display(fmt = "Error that occur during reading payload: {}", _0)]
    Payload(PayloadError),
    /// Line limit exceeded.
    #[display(fmt = "Line limit exceeded")]
    LimitOverflow,
    /// ContentType error.
    #[display(fmt = "Content-type error")]
    ContentTypeError(ContentTypeError),
}

/// Return `BadRequest` for `ReadlinesError`
impl ResponseError for ReadlinesError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ReadlinesError::LimitOverflow => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

/// A set of error that can occure during parsing content type
#[derive(PartialEq, Debug, Display)]
pub enum ContentTypeError {
    /// Can not parse content type
    #[display(fmt = "Can not parse content type")]
    ParseError,
    /// Unknown content encoding
    #[display(fmt = "Unknown content encoding")]
    UnknownEncoding,
}

/// Return `BadRequest` for `ContentTypeError`
impl ResponseError for ContentTypeError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Display)]
#[display(fmt = "UnknownError")]
struct UnitError;

/// `InternalServerError` for `UnitError`
impl ResponseError for UnitError {}

/// `InternalServerError` for `JsonError`
impl ResponseError for JsonError {}

/// `InternalServerError` for `FormError`
impl ResponseError for FormError {}

/// A set of errors that can occur during parsing HTTP streams
#[derive(Debug, Display)]
pub enum ParseError {
    /// An invalid `Method`, such as `GE.T`.
    #[display(fmt = "Invalid Method specified")]
    Method,
    /// An invalid `Uri`, such as `exam ple.domain`.
    #[display(fmt = "Uri error: {}", _0)]
    Uri(InvalidUri),
    /// An invalid `HttpVersion`, such as `HTP/1.1`
    #[display(fmt = "Invalid HTTP version specified")]
    Version,
    /// An invalid `Header`.
    #[display(fmt = "Invalid Header provided")]
    Header,
    /// A message head is too large to be reasonable.
    #[display(fmt = "Message head is too large")]
    TooLarge,
    /// A message reached EOF, but is not complete.
    #[display(fmt = "Message is incomplete")]
    Incomplete,
    /// An invalid `Status`, such as `1337 ELITE`.
    #[display(fmt = "Invalid Status provided")]
    Status,
    /// A timeout occurred waiting for an IO event.
    #[allow(dead_code)]
    #[display(fmt = "Timeout")]
    Timeout,
    /// An `io::Error` that occurred while trying to read or write to a network
    /// stream.
    #[display(fmt = "IO error: {}", _0)]
    Io(io::Error),
    /// Parsing a field as string failed
    #[display(fmt = "UTF8 error: {}", _0)]
    Utf8(Utf8Error),
}

/// Return `BadRequest` for `ParseError`
impl ResponseError for ParseError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::Io(err)
    }
}

impl From<InvalidUri> for ParseError {
    fn from(err: InvalidUri) -> ParseError {
        ParseError::Uri(err)
    }
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> ParseError {
        ParseError::Utf8(err)
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> ParseError {
        ParseError::Utf8(err.utf8_error())
    }
}

/// Helper type that can wrap any error and generate custom response.
///
/// In following example any `io::Error` will be converted into "BAD REQUEST"
/// response as opposite to *INTERNAL SERVER ERROR* which is defined by
/// default.
///
/// ```rust
/// # use std::io;
/// # use rweb::*;
///
/// fn index(req: Req) -> Result<&'static str> {
///     Err(error::ErrorBadRequest(io::Error::new(io::ErrorKind::Other, "error")))
/// }
/// ```
pub struct InternalError<T> {
    cause: T,
    status: InternalErrorType,
}

enum InternalErrorType {
    Status(StatusCode),
    Response(RefCell<Option<Resp>>),
}

impl<T> InternalError<T> {
    /// Create `InternalError` instance
    pub fn new(cause: T, status: StatusCode) -> Self {
        InternalError {
            cause,
            status: InternalErrorType::Status(status),
        }
    }

    /// Create `InternalError` with predefined `Response`.
    pub fn from_response(cause: T, response: Resp) -> Self {
        InternalError {
            cause,
            status: InternalErrorType::Response(RefCell::new(Some(response))),
        }
    }
}

impl<T> fmt::Debug for InternalError<T>
where
    T: fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.cause, f)
    }
}

impl<T> fmt::Display for InternalError<T>
where
    T: fmt::Display + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, f)
    }
}

impl<T> ResponseError for InternalError<T>
where
    T: fmt::Debug + fmt::Display + 'static,
{
    fn status_code(&self) -> StatusCode {
        match self.status {
            InternalErrorType::Status(st) => st,
            InternalErrorType::Response(ref resp) => {
                if let Some(resp) = resp.borrow().as_ref() {
                    resp.status()
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }

    fn error_response(&self) -> Resp {
        match self.status {
            InternalErrorType::Status(st) => {
                use std::fmt::Write;
                let mut res = Resp::new(st);
                let mut buf = BytesMut::new();
                let _ = write!(&mut buf, "{}", self);
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("text/plain; charset=utf-8"),
                );
                res.set_body(Body::from(buf));
                res
            }
            InternalErrorType::Response(ref resp) => {
                if let Some(resp) = resp.borrow_mut().take() {
                    resp
                } else {
                    Resp::new(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}

/// Helper function that creates wrapper of any error and generate *BAD
/// REQUEST* response.
#[allow(non_snake_case)]
pub fn ErrorBadRequest<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::BAD_REQUEST).into()
}

/// Helper function that creates wrapper of any error and generate
/// *UNAUTHORIZED* response.
#[allow(non_snake_case)]
pub fn ErrorUnauthorized<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::UNAUTHORIZED).into()
}

/// Helper function that creates wrapper of any error and generate
/// *PAYMENT_REQUIRED* response.
#[allow(non_snake_case)]
pub fn ErrorPaymentRequired<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::PAYMENT_REQUIRED).into()
}

/// Helper function that creates wrapper of any error and generate *FORBIDDEN*
/// response.
#[allow(non_snake_case)]
pub fn ErrorForbidden<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::FORBIDDEN).into()
}

/// Helper function that creates wrapper of any error and generate *NOT FOUND*
/// response.
#[allow(non_snake_case)]
pub fn ErrorNotFound<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::NOT_FOUND).into()
}

/// Helper function that creates wrapper of any error and generate *METHOD NOT
/// ALLOWED* response.
#[allow(non_snake_case)]
pub fn ErrorMethodNotAllowed<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::METHOD_NOT_ALLOWED).into()
}

/// Helper function that creates wrapper of any error and generate *NOT
/// ACCEPTABLE* response.
#[allow(non_snake_case)]
pub fn ErrorNotAcceptable<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::NOT_ACCEPTABLE).into()
}

/// Helper function that creates wrapper of any error and generate *PROXY
/// AUTHENTICATION REQUIRED* response.
#[allow(non_snake_case)]
pub fn ErrorProxyAuthenticationRequired<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::PROXY_AUTHENTICATION_REQUIRED).into()
}

/// Helper function that creates wrapper of any error and generate *REQUEST
/// TIMEOUT* response.
#[allow(non_snake_case)]
pub fn ErrorRequestTimeout<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::REQUEST_TIMEOUT).into()
}

/// Helper function that creates wrapper of any error and generate *CONFLICT*
/// response.
#[allow(non_snake_case)]
pub fn ErrorConflict<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::CONFLICT).into()
}

/// Helper function that creates wrapper of any error and generate *GONE*
/// response.
#[allow(non_snake_case)]
pub fn ErrorGone<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::GONE).into()
}

/// Helper function that creates wrapper of any error and generate *LENGTH
/// REQUIRED* response.
#[allow(non_snake_case)]
pub fn ErrorLengthRequired<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::LENGTH_REQUIRED).into()
}

/// Helper function that creates wrapper of any error and generate
/// *PAYLOAD TOO LARGE* response.
#[allow(non_snake_case)]
pub fn ErrorPayloadTooLarge<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::PAYLOAD_TOO_LARGE).into()
}

/// Helper function that creates wrapper of any error and generate
/// *URI TOO LONG* response.
#[allow(non_snake_case)]
pub fn ErrorUriTooLong<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::URI_TOO_LONG).into()
}

/// Helper function that creates wrapper of any error and generate
/// *UNSUPPORTED MEDIA TYPE* response.
#[allow(non_snake_case)]
pub fn ErrorUnsupportedMediaType<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::UNSUPPORTED_MEDIA_TYPE).into()
}

/// Helper function that creates wrapper of any error and generate
/// *RANGE NOT SATISFIABLE* response.
#[allow(non_snake_case)]
pub fn ErrorRangeNotSatisfiable<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::RANGE_NOT_SATISFIABLE).into()
}

/// Helper function that creates wrapper of any error and generate
/// *IM A TEAPOT* response.
#[allow(non_snake_case)]
pub fn ErrorImATeapot<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::IM_A_TEAPOT).into()
}

/// Helper function that creates wrapper of any error and generate
/// *MISDIRECTED REQUEST* response.
#[allow(non_snake_case)]
pub fn ErrorMisdirectedRequest<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::MISDIRECTED_REQUEST).into()
}

/// Helper function that creates wrapper of any error and generate
/// *UNPROCESSABLE ENTITY* response.
#[allow(non_snake_case)]
pub fn ErrorUnprocessableEntity<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::UNPROCESSABLE_ENTITY).into()
}

/// Helper function that creates wrapper of any error and generate
/// *LOCKED* response.
#[allow(non_snake_case)]
pub fn ErrorLocked<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::LOCKED).into()
}

/// Helper function that creates wrapper of any error and generate
/// *FAILED DEPENDENCY* response.
#[allow(non_snake_case)]
pub fn ErrorFailedDependency<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::FAILED_DEPENDENCY).into()
}

/// Helper function that creates wrapper of any error and generate
/// *UPGRADE REQUIRED* response.
#[allow(non_snake_case)]
pub fn ErrorUpgradeRequired<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::UPGRADE_REQUIRED).into()
}

/// Helper function that creates wrapper of any error and generate
/// *PRECONDITION FAILED* response.
#[allow(non_snake_case)]
pub fn ErrorPreconditionFailed<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::PRECONDITION_FAILED).into()
}

/// Helper function that creates wrapper of any error and generate
/// *PRECONDITION REQUIRED* response.
#[allow(non_snake_case)]
pub fn ErrorPreconditionRequired<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::PRECONDITION_REQUIRED).into()
}

/// Helper function that creates wrapper of any error and generate
/// *TOO MANY REQUESTS* response.
#[allow(non_snake_case)]
pub fn ErrorTooManyRequests<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::TOO_MANY_REQUESTS).into()
}

/// Helper function that creates wrapper of any error and generate
/// *REQUEST HEADER FIELDS TOO LARGE* response.
#[allow(non_snake_case)]
pub fn ErrorRequestHeaderFieldsTooLarge<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE).into()
}

/// Helper function that creates wrapper of any error and generate
/// *UNAVAILABLE FOR LEGAL REASONS* response.
#[allow(non_snake_case)]
pub fn ErrorUnavailableForLegalReasons<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS).into()
}

/// Helper function that creates wrapper of any error and generate
/// *EXPECTATION FAILED* response.
#[allow(non_snake_case)]
pub fn ErrorExpectationFailed<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::EXPECTATION_FAILED).into()
}

/// Helper function that creates wrapper of any error and
/// generate *INTERNAL SERVER ERROR* response.
#[allow(non_snake_case)]
pub fn ErrorInternalServerError<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::INTERNAL_SERVER_ERROR).into()
}

/// Helper function that creates wrapper of any error and
/// generate *NOT IMPLEMENTED* response.
#[allow(non_snake_case)]
pub fn ErrorNotImplemented<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::NOT_IMPLEMENTED).into()
}

/// Helper function that creates wrapper of any error and
/// generate *BAD GATEWAY* response.
#[allow(non_snake_case)]
pub fn ErrorBadGateway<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::BAD_GATEWAY).into()
}

/// Helper function that creates wrapper of any error and
/// generate *SERVICE UNAVAILABLE* response.
#[allow(non_snake_case)]
pub fn ErrorServiceUnavailable<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::SERVICE_UNAVAILABLE).into()
}

/// Helper function that creates wrapper of any error and
/// generate *GATEWAY TIMEOUT* response.
#[allow(non_snake_case)]
pub fn ErrorGatewayTimeout<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::GATEWAY_TIMEOUT).into()
}

/// Helper function that creates wrapper of any error and
/// generate *HTTP VERSION NOT SUPPORTED* response.
#[allow(non_snake_case)]
pub fn ErrorHttpVersionNotSupported<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::HTTP_VERSION_NOT_SUPPORTED).into()
}

/// Helper function that creates wrapper of any error and
/// generate *VARIANT ALSO NEGOTIATES* response.
#[allow(non_snake_case)]
pub fn ErrorVariantAlsoNegotiates<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::VARIANT_ALSO_NEGOTIATES).into()
}

/// Helper function that creates wrapper of any error and
/// generate *INSUFFICIENT STORAGE* response.
#[allow(non_snake_case)]
pub fn ErrorInsufficientStorage<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::INSUFFICIENT_STORAGE).into()
}

/// Helper function that creates wrapper of any error and
/// generate *LOOP DETECTED* response.
#[allow(non_snake_case)]
pub fn ErrorLoopDetected<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::LOOP_DETECTED).into()
}

/// Helper function that creates wrapper of any error and
/// generate *NOT EXTENDED* response.
#[allow(non_snake_case)]
pub fn ErrorNotExtended<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::NOT_EXTENDED).into()
}

/// Helper function that creates wrapper of any error and
/// generate *NETWORK AUTHENTICATION REQUIRED* response.
#[allow(non_snake_case)]
pub fn ErrorNetworkAuthenticationRequired<T>(err: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    InternalError::new(err, StatusCode::NETWORK_AUTHENTICATION_REQUIRED).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoded_error() {
        let resp: HttpResponse = UrlencodedError::Overflow { size: 0, limit: 0 }.error_response();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let resp: HttpResponse = UrlencodedError::UnknownLength.error_response();
        assert_eq!(resp.status(), StatusCode::LENGTH_REQUIRED);
        let resp: HttpResponse = UrlencodedError::ContentType.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_json_payload_error() {
        let resp: HttpResponse = JsonPayloadError::Overflow.error_response();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let resp: HttpResponse = JsonPayloadError::ContentType.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_query_payload_error() {
        let resp: HttpResponse = QueryPayloadError::Deserialize(
            serde_urlencoded::from_str::<i32>("bad query").unwrap_err(),
        )
        .error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_readlines_error() {
        let resp: HttpResponse = ReadlinesError::LimitOverflow.error_response();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let resp: HttpResponse = ReadlinesError::EncodingError.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
