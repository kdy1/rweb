use serde::{de::DeserializeOwned, Deserialize};
use warp::{filters::BoxedFilter, Filter, Rejection};

pub trait Factory {
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

impl<T> Factory for Json<T>
where
    T: 'static + Send + DeserializeOwned,
{
    type Filter = BoxedFilter<(Json<T>,)>;

    fn new() -> Self::Filter {
        warp::body::json().boxed()
    }
}

pub fn factory<F>() -> F::Filter
where
    F: Factory,
{
    F::new()
}