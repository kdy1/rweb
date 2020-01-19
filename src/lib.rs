pub use self::{
    extract::FromRequest,
    http::{msg::HttpMessage, Req, Resp},
    response::Response,
    types::*,
};
use crate::error::Error;
pub use either::Either;
pub use rweb_macros::{delete, get, head, options, patch, post, put};
pub use tokio::{main, test};
pub use warp::{self, filters, reject, reply, serve, Filter};

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

pub mod data;
pub mod error;
mod extract;
pub mod handler;
pub mod http;
mod responder;
mod response;
mod types;
