pub use http::StatusCode;
pub use std::clone::Clone;
use std::convert::Infallible;
pub use tokio;
use warp::{any, Filter};

pub fn provider<T: Clone + Send + Sync>(
    data: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    any().map(move || data.clone())
}
