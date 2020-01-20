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

impl<T> FromRequest for Json<T>
where
    T: 'static + Send + DeserializeOwned,
{
    type Filter = BoxedFilter<(Json<T>,)>;

    fn new() -> Self::Filter {
        warp::body::json().boxed()
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
