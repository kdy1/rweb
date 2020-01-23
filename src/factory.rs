use serde::{de::DeserializeOwned, Deserialize};
use warp::{
    filters::{multipart, ws::Ws, BoxedFilter},
    Filter, Rejection,
};

pub trait FromRequest: Sized {
    /// Extract should be `(Self,),`
    type Filter: Filter<Error = Rejection>;

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

    fn new() -> Self::Filter {
        warp::body::json().boxed()
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
