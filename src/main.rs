#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;

mod proofofwork;
mod block;
mod blockchain;
mod cli;

use blockchain::Blockchain;
use cli::CLI;

use std::env;

fn main() {
    let mut bc = Blockchain::new();
    let args: Vec<String> = env::args().collect();
    let mut cli = CLI::new(&mut bc, &args);
    cli.run();
}
