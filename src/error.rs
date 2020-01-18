pub trait ResponseError {}

#[derive(Debug, From)]
pub struct Error {
    err: Box<dyn ResponseError>,
}
