//! A macro to convert a function to rweb handler.
//!
//! # Attribute on parameters
//!
//! ## `#[body]`
//! Parses request body
//
//! ## `#[form]`
//! Parses request body
//
//! ## `#[json]`
//! Parses request body.
//!
//! ## `#[query]`
//! Parses query string.
//!
//! ```rust
//! #[get("/")]
//! fn use_query(#[query] qs: String) -> String {
//!    qs
//! }
//! ```
//!
//! ## `#[filter = "path_to_fn"]`
//! Calls function.
//!
//! **Note**: If the callee returns `()`, you should use `()` as type of
//! parameter.
//!
//! ```rust
//! use std::num::NonZeroU16;
//!
//! #[get("/math/{num}")]
//! fn math(num: u16, #[filter = "div_by"] denom: NonZeroU16) -> impl Reply {
//!    rweb::reply::json(&Math {
//!        op: format!("{} / {}", num, denom),
//!        output: num / denom.get(),
//!    })
//! }
//!
//! fn div_by() -> impl Filter<Extract = (NonZeroU16,), Error = Rejection> +
//! Copy {    rweb::header::<u16>("div-by").and_then(|n: u16| async move {
//!        if let Some(denom) = NonZeroU16::new(n) {
//!            Ok(denom)
//!        } else {
//!            Err(reject::custom(DivideByZero))
//!        }
//!    })
//! }
//! ```

pub use rweb_macros::{delete, get, head, options, patch, post, put, router};
pub use warp::{self, *};

#[doc(hidden)]
pub mod rt;
