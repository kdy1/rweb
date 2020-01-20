//! A macro to convert a function to rweb handler.
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
//! Parses query string.
//! ```rust
//! #[get("/")]
//! fn ret_accept(#[header="accept"] accept: String) -> String {
//!     accept
//! }
//! ```
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
//! ## `[data]`
//!
//! ```rust
//! use futures::lock::Mutex;
//! use rweb::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
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
//! ```

pub use rweb_macros::{delete, get, head, options, patch, post, put, router};
pub use warp::{self, *};

#[doc(hidden)]
pub mod rt;
