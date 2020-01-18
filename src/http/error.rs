use crate::{error::ResponseError, http::StatusCode};
use derive_more::Display;
use std::io;

#[derive(Display, Debug)]
/// A set of errors that can occur during payload parsing
pub enum PayloadError {
    /// A payload reached EOF, but is not complete.
    #[display(
        fmt = "A payload reached EOF, but is not complete. With error: {:?}",
        _0
    )]
    Incomplete(Option<io::Error>),
    /// Content encoding stream corruption
    #[display(fmt = "Can not decode content-encoding.")]
    EncodingCorrupted,
    /// A payload reached size limit.
    #[display(fmt = "A payload reached size limit.")]
    Overflow,
    /// A payload length is unknown.
    #[display(fmt = "A payload length is unknown.")]
    UnknownLength,
    //    /// Http2 payload error
    //    #[display(fmt = "{}", _0)]
    //    Http2Payload(h2::Error),
    /// Io error
    #[display(fmt = "{}", _0)]
    Io(io::Error),
}

/// `PayloadError` returns two possible results:
///
/// - `Overflow` returns `PayloadTooLarge`
/// - Other errors returns `BadRequest`
impl ResponseError for PayloadError {
    fn status_code(&self) -> StatusCode {
        match *self {
            PayloadError::Overflow => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
