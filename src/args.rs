use std::path::PathBuf;

pub use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(required = true)]
    pub file: PathBuf,
}
