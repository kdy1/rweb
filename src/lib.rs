pub use self::{
    extract::FromRequest,
    http::{msg::HttpMessage, Req, Resp},
    types::*,
};
use crate::error::Error;
pub use either::Either;
pub use rweb_macros::{delete, get, head, options, patch, post, put};
pub use warp::{self, filters, reject, reply, serve, test, Filter};

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

pub mod actix_handler;
pub mod error;
mod extract;
pub mod http;
mod responder;
mod types;
