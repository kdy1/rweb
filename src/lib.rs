//! A macro to convert a function to rweb handler.
//!
//! All parameters should satisfy one of the following.
//!
//!   - Has a path parameter with same name.
//!   - Annotated with the annotations documented below.
//!   - Has a type which implements [FromRequest].
//!
//!
//! # Path parmeters
//!
//!
//! # Attributes on function item
//!
//! ## `#[herader("content-type", "applcation/json)]`
//!
//! Make a route matches only if value of the header matches provided value.
//!
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! #[header("accept", "*/*")]
//! fn routes() -> &'static str {
//!    "This route matches only if accept header is '*/*'"
//! }
//!
//! fn main() {
//!     serve(routes());
//! }
//! ```
//!
//! ## `#[cors]`
//!
//!
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! #[cors(origins("example.com"), max_age = 600)]
//! fn cors_1() -> String {
//!    unreachable!()
//! }
//!
//! #[get("/")]
//! #[cors(origins("example.com"), methods(get), max_age = 600)]
//! fn cors_2() -> String {
//!    unreachable!()
//! }
//!
//! #[get("/")]
//! #[cors(origins("*"), methods(get), max_age = 600)]
//! fn cors_3() -> String {
//!    unreachable!()
//! }
//!
//! #[get("/")]
//! #[cors(
//!     origins("*"),
//!     methods(get, post, patch, delete),
//!     headers("accept"),
//!     max_age = 600
//! )]
//! fn cors_4() -> String {
//!    unreachable!()
//! }
//! ```
//!
//! ## `#[body_size(max = "8192")]`
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! #[body_size(max = "8192")]
//! fn body_size() -> String {
//!    unreachable!()
//! }
//! ```
//!
//!
//! # Attributes on parameters
//!
//! ## `#[body]`
//!
//! Parses request body. Type is `bytes::Bytes`.
//! ```rust
//! use rweb::*;
//! use http::Error;
//! use bytes::Bytes;
//!
//! #[post("/body")]
//! fn body(#[body] body: Bytes) -> Result<String, Error> {
//!    let _ = body;
//!    Ok(String::new())
//! }
//!
//! fn main() {
//!     serve(body());
//! }
//! ```
//!
//! ## `#[form]`
//! Parses request body. `Content-Type` should be `x-www-form-urlencoded`.
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
//! fn form(#[form] body: LoginForm) -> String {
//!    String::from("Ok")
//! }
//!
//! fn main() {
//!     serve(form());
//! }
//! ```
//!
//! ## `#[json]`
//! Parses request body. `Content-Type` should be `application/json`.
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
//!
//! fn main() {
//!     serve(json());
//! }
//! ```
//!
//! Note that you can mix the order of parameters.
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
//! #[get("/param/{a}/{b}")]
//! fn body_between_path_params(a: u32, #[json] body: LoginForm, b: u32) ->
//! String {     assert_eq!(body.id, "TEST_ID");
//!     assert_eq!(body.password, "TEST_PASSWORD");
//!     (a + b).to_string()
//! }
//!
//! fn main() {
//!     serve(body_between_path_params());
//! }
//! ```
//!
//! ## `#[query]`
//!
//! Parses query string.
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! fn use_query(#[query] qs: String) -> String {
//!     qs
//! }
//!
//! fn main() {
//!     serve(use_query());
//! }
//! ```
//!
//! ## `#[header = "header-name"]`
//! Value of the header.
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! fn ret_accept(#[header = "accept"] accept: String) -> String {
//!     accept
//! }
//! fn main() {
//!     serve(ret_accept());
//! }
//! ```
//!
//! ## `#[cookie = "cookie-name"]`
//! Value of the header.
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! fn cookie(#[header = "sess"] sess_id: String) -> String {
//!     sess_id
//! }
//! fn main() {
//!     serve(cookie());
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
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Math {
//!     op: String,
//!     output: u16,
//! }
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
//!
//! #[derive(Debug)]
//! struct DivideByZero;
//!
//! impl reject::Reject for DivideByZero {}
//!
//! fn main() {
//!     serve(math());
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
//! ```rust
//! use http::StatusCode;
//! use rweb::{filters::BoxedFilter, *};
//!
//! impl FromRequest for User {
//!    type Filter = BoxedFilter<(User,)>;
//!
//!    fn new() -> Self::Filter {
//!        // In real world, you can use a header like Authorization
//!        header::<String>("x-user-id").map(|id| User { id }).boxed()
//!    }
//! }
//!
//!
//! #[derive(Schema)]
//! struct User {
//!    id: String,
//! }
//!
//! #[get("/")]
//! fn index(user: User) -> String {
//!    user.id
//! }
//!
//! fn main() {
//!     serve(index());
//! }
//! ```
//!
//!
//! # Guards
//! ```rust
//! use rweb::*;
//!
//! // This handler is invoked only if x-appengine-cron matches 1 (case insensitive).
//! #[get("/")]
//! #[header("X-AppEngine-Cron", "1")]
//! fn gae_cron() -> String {
//!     String::new()
//! }
//! ```
//!
//! # `#[router]`
//!
//! `#[router]` can be used to group routes.
//!
//! ## `#[data]`
//!
//! You can use `#[data]` with a router.
//! ```rust
//! use rweb::*;
//!
//! #[derive(Default, Clone)]
//! struct Db {}
//!
//! #[get("/use")]
//! fn use_db(#[data] _db: Db) -> String {
//!    String::new()
//! }
//!
//! #[router("/data", services(use_db))]
//! fn data_param(#[data] db: Db) {}
//! ```
//!
//!
//! ## Guard
//! ```rust
//! use rweb::*;
//!
//! #[get("/")]
//! fn admin_index() -> String {
//!    String::new()
//! }
//!
//! #[get("/users")]
//! fn admin_users() -> String {
//!    String::new()
//! }
//!
//! #[router("/admin", services(admin_index, admin_users))]
//! #[header("X-User-Admin", "1")]
//! fn admin() {}
//! ```

pub use self::factory::{Form, FromRequest, Json, Query};
pub use rweb_macros::{delete, get, head, options, patch, post, put, router, Schema};
pub use warp::{self, *};

mod factory;
#[cfg(feature = "openapi")]
pub mod openapi;
#[doc(hidden)]
pub mod rt;

pub mod routes;
pub use self::routes::*;
