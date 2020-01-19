use warp::Filter;

pub trait FilterTransform<Prev>
where
    Prev: Filter,
{
    type Output: Filter;

    fn transform(self, prev: Prev) -> Self::Output;
}

impl<F, Prev, Out> FilterTransform<Prev> for F
where
    F: FnOnce(Prev) -> Out,
    Prev: Filter,
    Out: Filter,
{
    type Output = Out;

    fn transform(self, prev: Prev) -> Self::Output {
        (self)(prev)
    }
}
