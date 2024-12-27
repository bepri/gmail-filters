//! Parsing module for command-line arguments.
use ::std::path::PathBuf;

#[doc(hidden)]
pub use ::clap::Parser;

/// Command-line arguments.
#[derive(Parser, Debug)]
pub struct Args {
    /// Configuration file of email filters. Supported formats: toml
    #[arg(required = true)]
    pub file: PathBuf,
}
