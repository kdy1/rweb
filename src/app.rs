use crate::warp_ext::FilterTransform;
use warp::Filter;

pub struct App<F>
where
    F: Filter,
{
    svc: F,
}

impl<F> App<F>
where
    F: Filter,
{
    pub fn new(svc: F) -> Self {
        App { svc }
    }

    pub fn add<T>(self, tr: T) -> App<T::Output>
    where
        T: FilterTransform<F>,
    {
        App {
            svc: tr.transform(self.svc),
        }
    }
}
