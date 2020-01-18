use derive_more::{Display, From};
use url::ParseError as UrlParseError;

pub trait ResponseError {}

/// Errors which can occur when attempting to generate resource uri.
#[derive(Debug, PartialEq, Display, From)]
pub enum UrlGenerationError {
    /// Resource not found
    #[display(fmt = "Resource not found")]
    ResourceNotFound,
    /// Not all path pattern covered
    #[display(fmt = "Not all path pattern covered")]
    NotEnoughElements,
    /// URL parse error
    #[display(fmt = "{}", _0)]
    ParseError(UrlParseError),
}

/// `InternalServerError` for `UrlGeneratorError`
impl ResponseError for UrlGenerationError {}

#[derive(From)]
pub struct Error {
    err: Box<dyn ResponseError>,
}
