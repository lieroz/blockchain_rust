#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate bs58;
extern crate serde;

mod block;
mod blockchain;
mod cli;
mod proofofwork;
mod transaction;
mod wallet;
mod wallets;

use cli::CLI;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cli = CLI::new(&args);
    cli.run();
}
