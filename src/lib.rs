//! A macro to convert a function to rweb handler.
//!
//! All parameters should satisfy one of the following.
//!
//!   - Has a path parameter with same name.
//!   - Annotated with the annotations documented below.
//!   - Has a type which implements [FromRequest].
//!
//! # Attribute on parameters
//!
//! ## `#[body]`
//! Parses request body
//! ```rust
//! use rweb::*;
//! use bytes::Bytes;
//!
//! #[post("/body")]
//! fn body(#[body] body: Bytes) -> Result<String, Error> {
//!    let _ = body;
//!    Ok(String::new())
//! }
//! ```
//!
//! ## `#[form]`
//! Parses request body
//!
//! ```rust
//! use rweb::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct LoginForm {
//!     id: String,
//!     password: String,
//! }
//!
//! #[post("/form")]
//! fn form(#[form] body: LoginForm) -> Result<String, Error> {
//!    Ok(serde_json::to_string(&body).unwrap())
//! }
//! ```
//!
//! ## `#[json]`
//! Parses request body.
//! ```rust
//! use rweb::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct LoginForm {
//!     id: String,
//!     password: String,
//! }
//!
//! #[post("/json")]
//! fn json(#[json] body: LoginForm) -> String {
//!     String::from("Ok")
//! }
//! ```
//!
//! Note that you can mix the order of parameters.
//! ```rust
//! #[get("/param/{a}/{b}")]
//! fn body_between_path_params(a: u32, #[json] body: LoginForm, b: u32) ->
//! String {     assert_eq!(body.id, "TEST_ID");
//!     assert_eq!(body.password, "TEST_PASSWORD");
//!     (a + b).to_string()
//! }
//! ```
//!
//! ## `#[query]`
//! Parses query string.
//! ```rust
//! #[get("/")]
//! fn use_query(#[query] qs: String) -> String {
//!     qs
//! }
//! ```
//!
//! ## `#[header]`
//! Value of the header.
//! ```rust
//! #[get("/")]
//! fn ret_accept(#[header = "accept"] accept: String) -> String {
//!     accept
//! }
//! ```
//!
//! ### Using header as a guard
//!
//! ```rust
//! use rweb::*;
//! use std::net::SocketAddr;
//!
//! #[get("/")]
//! fn routes(#[header(accept = "*/*")] _guard: (), #[header = "host"] host: SocketAddr) -> String {
//!    format!("accepting stars on {}", host)
//! }
//! ```
//!
//!
//! ## `#[filter = "path_to_fn"]`
//! Calls function.
//!
//! **Note**: If the callee returns `()`, you should use `()` as type. (Type
//! alias is not allowed)
//! ```rust
//! use std::num::NonZeroU16;
//! use rweb::*;
//!
//! #[get("/math/{num}")]
//! fn math(num: u16, #[filter = "div_by"] denom: NonZeroU16) -> impl Reply {
//!     rweb::reply::json(&Math {
//!         op: format!("{} / {}", num, denom),
//!         output: num / denom.get(),
//!     })
//! }
//!
//! fn div_by() -> impl Filter<Extract = (NonZeroU16,), Error = Rejection> +Copy
//! {    rweb::header::<u16>("div-by").and_then(|n: u16| async move {
//!        if let Some(denom) = NonZeroU16::new(n) {
//!            Ok(denom)
//!        } else {
//!            Err(reject::custom(DivideByZero))
//!        }
//!    })
//! }
//! ```
//!
//! ## `#[data]`
//! ```rust
//! use futures::lock::Mutex;
//! use rweb::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone, Default)]
//! struct Db {
//!    items: Arc<Mutex<Vec<String>>>,
//! }
//!
//! #[get("/")]
//! async fn index(#[data] db: Db) -> Result<String, Rejection> {
//!    let items = db.items.lock().await;
//!
//!    Ok(items.len().to_string())
//! }
//!
//! fn main() {
//!     let db = Default::default();
//!     serve(index(db));
//! }
//! ```
//!
//! # FromRequest
//!
//! ```rust
//! use http::StatusCode;
//! use rweb::{filters::BoxedFilter, *};
//!
//! impl FromRequest for User {
//!    type Filter = BoxedFilter<(User,)>;
//!
//!    fn new() -> Self::Filter {
//!        header::<String>("x-user-id").map(|id| User { id }).boxed()
//!    }
//! }
//!
//! struct User {
//!    id: String,
//! }
//!
//! #[get("/")]
//! fn index(user: User) -> String {
//!    user.id
//! }
//! ```

pub use self::factory::{FromRequest, Json};
pub use rweb_macros::{delete, get, head, options, patch, post, put, router};
pub use warp::{self, *};

mod factory;
#[doc(hidden)]
pub mod rt;
