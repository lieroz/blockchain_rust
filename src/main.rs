#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate bs58;
extern crate serde;

mod block;
mod blockchain;
mod cli;
mod merkle_tree;
mod proofofwork;
mod transaction;
mod utxo_set;
mod wallet;
mod wallets;
mod server;

use cli::CLI;
use server::Server;

use std::env;

fn main() {
    Server::start("3001", "asdf");

    let args: Vec<String> = env::args().collect();
    let cli = CLI::new(&args);
    cli.run();
}
