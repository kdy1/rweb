#![deny(warnings)]

use rweb::{get, http::StatusCode, reject, Filter, Rejection, Reply};
use serde::Serialize;
use std::{convert::Infallible, num::NonZeroU16};

#[get("/math/{num}")]
fn math(num: u16, #[filter = "div_by"] denom: NonZeroU16) -> impl Reply {
    rweb::reply::json(&Math {
        op: format!("{} / {}", num, denom),
        output: num / denom.get(),
    })
}

/// Rejections represent cases where a filter should not continue processing
/// the request, but a different filter *could* process it.
#[tokio::main]
async fn main() {
    let routes = rweb::get().and(math()).recover(handle_rejection);

    rweb::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

/// Extract a denominator from a "div-by" header, or reject with DivideByZero.
fn div_by() -> impl Filter<Extract = (NonZeroU16,), Error = Rejection> + Copy {
    rweb::header::<u16>("div-by").and_then(|n: u16| async move {
        if let Some(denom) = NonZeroU16::new(n) {
            Ok(denom)
        } else {
            Err(reject::custom(DivideByZero))
        }
    })
}

#[derive(Debug)]
struct DivideByZero;

impl reject::Reject for DivideByZero {}

// JSON replies

/// A successful math operation.
#[derive(Serialize)]
struct Math {
    op: String,
    output: u16,
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(DivideByZero) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "DIVIDE_BY_ZERO";
    } else if err.find::<rweb::reject::MethodNotAllowed>().is_some() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = rweb::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(rweb::reply::with_status(json, code))
}
