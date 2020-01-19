pub use self::{
    app::App,
    extract::FromRequest,
    http::{msg::HttpMessage, Req, Resp},
    response::Response,
    types::*,
};
use crate::error::Error;
pub use either::Either;
pub use rweb_macros::{delete, get, head, options, patch, post, put};
pub use tokio::{main, test};
pub use warp::{self, filters};

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

mod app;
pub mod data;
pub mod encoding;
pub mod error;
mod extract;
pub mod guard;
pub mod handler;
pub mod http;
mod responder;
mod response;
pub mod route;
pub mod service;
mod types;
mod util;
pub mod warp_ext;
