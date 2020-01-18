//! Middlewares

pub use self::{
    condition::Condition, defaultheaders::DefaultHeaders, logger::Logger, normalize::NormalizePath,
};

#[cfg(feature = "compress")]
mod compress;
#[cfg(feature = "compress")]
pub use self::compress::Compress;

mod condition;
mod defaultheaders;
pub mod errhandlers;
mod logger;
mod normalize;
