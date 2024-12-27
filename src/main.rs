mod args;
mod filters;

use args::{Args, Parser};
use filters::parsing::get_config;

fn main() {
    let args = Args::parse();
    let config = get_config(args.file);
    println!("{config:#?}");
}
