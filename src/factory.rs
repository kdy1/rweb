use serde::{de::DeserializeOwned, Deserialize, Serialize};
use warp::{
    filters::{multipart, ws::Ws, BoxedFilter},
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

    fn new() -> Self::Filter;
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Json<T>(T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Json<T> {
    #[inline(always)]
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

#[derive(Deserialize)]
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

    fn new() -> Self::Filter {
        warp::body::form().boxed()
    }
}

#[derive(Deserialize)]
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

impl FromRequest for Ws {
    type Filter = BoxedFilter<(Ws,)>;

    fn new() -> Self::Filter {
        warp::ws().boxed()
    }
}

impl FromRequest for multipart::FormData {
    type Filter = BoxedFilter<(Self,)>;

    fn new() -> Self::Filter {
        warp::multipart::form().boxed()
    }
}
