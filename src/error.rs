use derive_more::From;

pub trait ResponseError {}

#[derive(From)]
pub struct Error {
    err: Box<dyn ResponseError>,
}
