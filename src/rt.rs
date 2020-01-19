use crate::Filter;
use warp::Rejection;

pub fn wrap_typed<F: Filter>(
    handler: F,
) -> impl Filter<Extract = impl crate::reply::Reply, Error = Rejection> {
}
