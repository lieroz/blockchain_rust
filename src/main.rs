#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;

mod proofofwork;
mod block;
mod blockchain;
mod cli;
mod transaction;

use cli::CLI;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cli = CLI::new(&args);
    cli.run();
}
