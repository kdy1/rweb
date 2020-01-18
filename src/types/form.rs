//! Form extractor

#[cfg(feature = "compress")]
use crate::dev::Decompress;
use crate::{
    error::{Error, UrlencodedError},
    extract::FromRequest,
    http::{Payload, StatusCode},
    responder::Responder,
    Req, Resp,
};
use bytes::BytesMut;
use encoding_rs::{Encoding, UTF_8};
use futures::{
    future::{err, ok, FutureExt, LocalBoxFuture, Ready},
    StreamExt,
};
use hyper::header::CONTENT_LENGTH;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt,
    future::Future,
    ops,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

/// Form data helper (`application/x-www-form-urlencoded`)
///
/// Can be use to extract url-encoded data from the request body,
/// or send url-encoded data as the response.
///
/// ## Extract
///
/// To extract typed information from request's body, the type `T` must
/// implement the `Deserialize` trait from *serde*.
///
/// [**FormConfig**](struct.FormConfig.html) allows to configure extraction
/// process.
///
/// ### Example
/// ```rust
/// use rweb::web;
/// use serde_derive::Deserialize;
///
/// #[derive(Deserialize)]
/// struct FormData {
///     username: String,
/// }
///
/// /// Extract form data using serde.
/// /// This handler get called only if content type is *x-www-form-urlencoded*
/// /// and content of the request could be deserialized to a `FormData` struct
/// fn index(form: web::Form<FormData>) -> String {
///     format!("Welcome {}!", form.username)
/// }
/// # fn main() {}
/// ```
///
/// ## Respond
///
/// The `Form` type also allows you to respond with well-formed url-encoded
/// data: simply return a value of type Form<T> where T is the type to be
/// url-encoded. The type  must implement `serde::Serialize`;
///
/// ### Example
/// ```rust
/// use rweb::*;
/// use serde_derive::Serialize;
///
/// #[derive(Serialize)]
/// struct SomeForm {
///     name: String,
///     age: u8
/// }
///
/// // Will return a 200 response with header
/// // `Content-Type: application/x-www-form-urlencoded`
/// // and body "name=actix&age=123"
/// fn index() -> web::Form<SomeForm> {
///     web::Form(SomeForm {
///         name: "actix".into(),
///         age: 123
///     })
/// }
/// # fn main() {}
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Form<T>(pub T);

impl<T> Form<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Error>>;
    type Config = FormConfig;

    #[inline]
    fn from_request(req: &Req, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let (limit, err) = req
            .app_data::<FormConfig>()
            .map(|c| (c.limit, c.ehandler.clone()))
            .unwrap_or((16384, None));

        UrlEncoded::new(req, payload)
            .limit(limit)
            .map(move |res| match res {
                Err(e) => {
                    if let Some(err) = err {
                        Err((*err)(e, &req2))
                    } else {
                        Err(e.into())
                    }
                }
                Ok(item) => Ok(Form(item)),
            })
            .boxed_local()
    }
}

impl<T: fmt::Debug> fmt::Debug for Form<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Form<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Serialize> Responder for Form<T> {
    type Error = Error;
    type Future = Ready<Result<Resp, Error>>;

    fn respond_to(self, _: &Req) -> Self::Future {
        let body = match serde_urlencoded::to_string(&self.0) {
            Ok(body) => body,
            Err(e) => return err(e.into()),
        };

        ok(Resp::builder(StatusCode::OK)
            .content_type("x-www-form-urlencoded")
            .body(body.into())
            .build())
    }
}

/// Form extractor configuration
///
/// ```rust
/// use rweb::{web, App, FromRequest, Result};
/// use serde_derive::Deserialize;
///
/// #[derive(Deserialize)]
/// struct FormData {
///     username: String,
/// }
///
/// /// Extract form data using serde.
/// /// Custom configuration is used for this handler, max payload size is 4k
/// async fn index(form: web::Form<FormData>) -> Result<String> {
///     Ok(format!("Welcome {}!", form.username))
/// }
///
/// fn main() {
///     let app = App::new().service(
///         web::resource("/index.html")
///             // change `Form` extractor configuration
///             .app_data(
///                 web::Form::<FormData>::configure(|cfg| cfg.limit(4097))
///             )
///             .route(web::get().to(index))
///     );
/// }
/// ```
#[derive(Clone)]
pub struct FormConfig {
    limit: usize,
    ehandler: Option<Rc<dyn Fn(UrlencodedError, &Req) -> Error>>,
}

impl FormConfig {
    /// Change max size of payload. By default max size is 16Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(UrlencodedError, &Req) -> Error + 'static,
    {
        self.ehandler = Some(Rc::new(f));
        self
    }
}

impl Default for FormConfig {
    fn default() -> Self {
        FormConfig {
            limit: 16384,
            ehandler: None,
        }
    }
}

/// Future that resolves to a parsed urlencoded values.
///
/// Parse `application/x-www-form-urlencoded` encoded request's body.
/// Return `UrlEncoded` future. Form can be deserialized to any type that
/// implements `Deserialize` trait from *serde*.
///
/// Returns error:
///
/// * content type is not `application/x-www-form-urlencoded`
/// * content-length is greater than 32k
pub struct UrlEncoded<U> {
    #[cfg(feature = "compress")]
    stream: Option<Decompress<Payload>>,
    #[cfg(not(feature = "compress"))]
    stream: Option<Payload>,
    limit: usize,
    length: Option<usize>,
    encoding: &'static Encoding,
    err: Option<UrlencodedError>,
    fut: Option<LocalBoxFuture<'static, Result<U, UrlencodedError>>>,
}

impl<U> UrlEncoded<U> {
    /// Create a new future to URL encode a request
    pub fn new(req: &Req, payload: &mut Payload) -> UrlEncoded<U> {
        // check content type
        if req.content_type().to_lowercase() != "application/x-www-form-urlencoded" {
            return Self::err(UrlencodedError::ContentType);
        }
        let encoding = match req.encoding() {
            Ok(enc) => enc,
            Err(_) => return Self::err(UrlencodedError::ContentType),
        };

        let mut len = None;
        if let Some(l) = req.headers().get(&CONTENT_LENGTH) {
            if let Ok(s) = l.to_str() {
                if let Ok(l) = s.parse::<usize>() {
                    len = Some(l)
                } else {
                    return Self::err(UrlencodedError::UnknownLength);
                }
            } else {
                return Self::err(UrlencodedError::UnknownLength);
            }
        };

        #[cfg(feature = "compress")]
        let payload = Decompress::from_headers(payload.take(), req.headers());
        #[cfg(not(feature = "compress"))]
        let payload = payload.take();

        UrlEncoded {
            encoding,
            stream: Some(payload),
            limit: 32_768,
            length: len,
            fut: None,
            err: None,
        }
    }

    fn err(e: UrlencodedError) -> Self {
        UrlEncoded {
            stream: None,
            limit: 32_768,
            fut: None,
            err: Some(e),
            length: None,
            encoding: UTF_8,
        }
    }

    /// Change max size of payload. By default max size is 256Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl<U> Future for UrlEncoded<U>
where
    U: DeserializeOwned + 'static,
{
    type Output = Result<U, UrlencodedError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref mut fut) = self.fut {
            return Pin::new(fut).poll(cx);
        }

        if let Some(err) = self.err.take() {
            return Poll::Ready(Err(err));
        }

        // payload size
        let limit = self.limit;
        if let Some(len) = self.length.take() {
            if len > limit {
                return Poll::Ready(Err(UrlencodedError::Overflow { size: len, limit }));
            }
        }

        // future
        let encoding = self.encoding;
        let mut stream = self.stream.take().unwrap();

        self.fut = Some(
            async move {
                let mut body = BytesMut::with_capacity(8192);

                while let Some(item) = stream.next().await {
                    let chunk = item?;
                    if (body.len() + chunk.len()) > limit {
                        return Err(UrlencodedError::Overflow {
                            size: body.len() + chunk.len(),
                            limit,
                        });
                    } else {
                        body.extend_from_slice(&chunk);
                    }
                }

                if encoding == UTF_8 {
                    serde_urlencoded::from_bytes::<U>(&body).map_err(|_| UrlencodedError::Parse)
                } else {
                    let body = encoding
                        .decode_without_bom_handling_and_without_replacement(&body)
                        .map(|s| s.into_owned())
                        .ok_or(UrlencodedError::Parse)?;
                    serde_urlencoded::from_str::<U>(&body).map_err(|_| UrlencodedError::Parse)
                }
            }
            .boxed_local(),
        );
        self.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        http::header::{HeaderValue, CONTENT_TYPE},
        test::TestRequest,
    };

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Info {
        hello: String,
        counter: i64,
    }

    #[tokio::test]
    async fn test_form() {
        let (req, mut pl) =
            TestRequest::with_header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(CONTENT_LENGTH, "11")
                .set_payload(Bytes::from_static(b"hello=world&counter=123"))
                .to_http_parts();

        let Form(s) = Form::<Info>::from_request(&req, &mut pl).await.unwrap();
        assert_eq!(
            s,
            Info {
                hello: "world".into(),
                counter: 123
            }
        );
    }

    fn eq(err: UrlencodedError, other: UrlencodedError) -> bool {
        match err {
            UrlencodedError::Overflow { .. } => match other {
                UrlencodedError::Overflow { .. } => true,
                _ => false,
            },
            UrlencodedError::UnknownLength => match other {
                UrlencodedError::UnknownLength => true,
                _ => false,
            },
            UrlencodedError::ContentType => match other {
                UrlencodedError::ContentType => true,
                _ => false,
            },
            _ => false,
        }
    }

    #[tokio::test]
    async fn test_urlencoded_error() {
        let (req, mut pl) =
            TestRequest::with_header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(CONTENT_LENGTH, "xxxx")
                .to_http_parts();
        let info = UrlEncoded::<Info>::new(&req, &mut pl).await;
        assert!(eq(info.err().unwrap(), UrlencodedError::UnknownLength));

        let (req, mut pl) =
            TestRequest::with_header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(CONTENT_LENGTH, "1000000")
                .to_http_parts();
        let info = UrlEncoded::<Info>::new(&req, &mut pl).await;
        assert!(eq(
            info.err().unwrap(),
            UrlencodedError::Overflow { size: 0, limit: 0 }
        ));

        let (req, mut pl) = TestRequest::with_header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, "10")
            .to_http_parts();
        let info = UrlEncoded::<Info>::new(&req, &mut pl).await;
        assert!(eq(info.err().unwrap(), UrlencodedError::ContentType));
    }

    #[tokio::test]
    async fn test_urlencoded() {
        let (req, mut pl) =
            TestRequest::with_header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(CONTENT_LENGTH, "11")
                .set_payload(Bytes::from_static(b"hello=world&counter=123"))
                .to_http_parts();

        let info = UrlEncoded::<Info>::new(&req, &mut pl).await.unwrap();
        assert_eq!(
            info,
            Info {
                hello: "world".to_owned(),
                counter: 123
            }
        );

        let (req, mut pl) = TestRequest::with_header(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=utf-8",
        )
        .header(CONTENT_LENGTH, "11")
        .set_payload(Bytes::from_static(b"hello=world&counter=123"))
        .to_http_parts();

        let info = UrlEncoded::<Info>::new(&req, &mut pl).await.unwrap();
        assert_eq!(
            info,
            Info {
                hello: "world".to_owned(),
                counter: 123
            }
        );
    }

    #[tokio::test]
    async fn test_responder() {
        let req = TestRequest::default().to_http_request();

        let form = Form(Info {
            hello: "world".to_string(),
            counter: 123,
        });
        let resp = form.respond_to(&req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("application/x-www-form-urlencoded")
        );

        use crate::responder::tests::BodyTest;
        assert_eq!(resp.body().bin_ref(), b"hello=world&counter=123");
    }
}
