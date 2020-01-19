pub use self::{
    app::App,
    extract::FromRequest,
    http::{msg::HttpMessage, Req, Resp},
    response::Response,
    types::*,
};
use crate::error::Error;
pub use either::Either;
pub use rweb_macros::{connect, delete, get, head, options, patch, post, put, trace};
pub use rweb_router::Path;
pub use tokio::{main, test};

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

mod app;
pub mod data;
pub mod encoding;
pub mod error;
mod extract;
pub mod guard;
pub mod handler;
pub mod http;
pub mod resource;
mod responder;
mod response;
pub mod rmap;
pub mod route;
pub mod service;
mod types;
mod util;
