pub use http::StatusCode;
use std::convert::Infallible;
pub use std::{borrow::Cow, clone::Clone, collections::BTreeMap, default::Default};
pub use tokio;
use warp::{any, Filter};

pub fn provider<T: Clone + Send + Sync>(
    data: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    any().map(move || data.clone())
}
