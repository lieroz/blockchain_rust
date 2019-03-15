#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;

mod proofofwork;
mod block;
mod blockchain;

use proofofwork::ProofOfWork;
use blockchain::Blockchain;

fn main() {
    let mut bc = Blockchain::new();

    bc.add_block("Send 1 BTC to Ivan");
    bc.add_block("Send 2 more BTC to Ivan");

    for block in bc.iter() {
        println!("{:?}", block);
        let pow = ProofOfWork::new(&block);
        println!("{}", pow.validate());
    }
}
