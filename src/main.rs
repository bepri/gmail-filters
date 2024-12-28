#![warn(missing_docs, clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

mod args;
mod filters;

use args::{Args, Parser};
use filters::parsing::get_config;

use ::std::process::ExitCode;

fn main() -> Result<ExitCode, Box<dyn ::std::error::Error>> {
    let args = Args::parse();
    let file_content = ::std::fs::read_to_string(args.file)?;
    let config = get_config(file_content);
    println!("{config:#?}");

    Ok(ExitCode::SUCCESS)
}
