//! Some common definitions to be used across the crate.

/// A generic error type
pub type Error = Box<dyn ::std::error::Error>;

/// A generic result type
pub type Result<T> = ::std::result::Result<T, Error>;
