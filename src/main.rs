#![warn(missing_docs, clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

mod args;
mod filters;

use args::{Args, Parser};
use filters::parsing::get_config;

fn main() {
    let args = Args::parse();
    let config = get_config(args.file);
    println!("{config:#?}");
}
