use futures::future::ok;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "multipart")]
use warp::filters::multipart;
#[cfg(feature = "websocket")]
use warp::filters::ws::Ws;
use warp::{
    filters::BoxedFilter,
    reply::{json, Response},
    Filter, Rejection, Reply,
};

pub trait FromRequest: Sized {
    /// Extract should be `(Self,),`
    type Filter: Filter<Error = Rejection>;

    /// It's true iff the type represents whole request body.
    ///
    /// It returns true for `Json<T>` and `Form<T>`.
    fn is_body() -> bool {
        false
    }

    /// It's true if the type is optional.
    ///
    /// It returns true for `Option<T>`.
    fn is_optional() -> bool {
        false
    }

    /// It's true iff the type represents whole request query.
    ///
    /// It returns true for `Query<T>`.
    fn is_query() -> bool {
        false
    }

    fn content_type() -> &'static str {
        "*/*"
    }

    fn new() -> Self::Filter;
}

impl<T> FromRequest for Option<T>
where
    T: 'static + FromRequest + Send + Send,
    T::Filter: Send + Sync + Filter<Extract = (T,), Error = Rejection>,
{
    type Filter = BoxedFilter<(Option<T>,)>;

    fn is_body() -> bool {
        T::is_body()
    }

    fn is_optional() -> bool {
        true
    }

    fn is_query() -> bool {
        T::is_query()
    }

    fn new() -> Self::Filter {
        T::new()
            .map(Some)
            .or_else(|_| ok::<_, Rejection>((None,)))
            .boxed()
    }
}

/// Represents request body or response.
///
///
/// If it is in a parameter, content-type should be `application/json` and
/// request body will be deserialized.
///
/// If it is in a return type, response will contain a content type header with
/// value `application/json`, and value is serialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Json<T>(T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(v: T) -> Self {
        Json(v)
    }
}

impl<T> FromRequest for Json<T>
where
    T: 'static + Send + DeserializeOwned,
{
    type Filter = BoxedFilter<(Json<T>,)>;

    fn is_body() -> bool {
        true
    }

    fn content_type() -> &'static str {
        "application/json"
    }

    fn new() -> Self::Filter {
        warp::body::json().boxed()
    }
}

impl<T> Reply for Json<T>
where
    T: Serialize + Send,
{
    fn into_response(self) -> Response {
        json(&self.0).into_response()
    }
}

/// Represents a request body with `www-url-form-encoded` content type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(transparent)]
pub struct Form<T>(T);

impl<T> Form<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for Form<T>
where
    T: 'static + Send + DeserializeOwned,
{
    type Filter = BoxedFilter<(Form<T>,)>;

    fn is_body() -> bool {
        true
    }

    fn content_type() -> &'static str {
        "x-www-form-urlencoded"
    }

    fn new() -> Self::Filter {
        warp::body::form().boxed()
    }
}

/// Represents all query parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(transparent)]
pub struct Query<T>(T);

impl<T> Query<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for Query<T>
where
    T: 'static + Send + DeserializeOwned,
{
    type Filter = BoxedFilter<(Query<T>,)>;

    fn is_query() -> bool {
        true
    }

    fn new() -> Self::Filter {
        warp::query().boxed()
    }
}

#[cfg(feature = "websocket")]
impl FromRequest for Ws {
    type Filter = BoxedFilter<(Ws,)>;

    fn new() -> Self::Filter {
        warp::ws().boxed()
    }
}

#[cfg(feature = "multipart")]
impl FromRequest for multipart::FormData {
    type Filter = BoxedFilter<(Self,)>;

    fn new() -> Self::Filter {
        warp::multipart::form().boxed()
    }
}
