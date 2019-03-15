use crate::blockchain::Blockchain;
use crate::proofofwork::ProofOfWork;

use std::process;

pub struct CLI<'a> {
    bc: &'a mut Blockchain,
    args: &'a[String],
}

impl<'a> CLI<'a> {
    pub fn new(bc: &'a mut Blockchain, args: &'a[String]) -> CLI<'a> {
        CLI{bc, args}
    }

    fn print_usage(&self) {
        println!("Usage:");
        println!("    add_block -data BLOCK_DATA - add a block to blockchain");
        println!("    print_chain - print all the blocks of the blockchain");
    }

    fn validate_args(&self) {
        if self.args.len() < 2 {
            self.print_usage();
            process::exit(1);
        }
    }

    fn add_block(&mut self, data: &str) {
        self.bc.add_block(data);
    }

    fn print_chain(&mut self) {
        for block in self.bc.iter() {
            println!("Prev. hash: {}", block.prev_block_hash());
            println!("Data: {}", block.data());
            println!("Hash: {}", block.hash());
            let pow = ProofOfWork::new(&block);
            println!("PoW: {}", pow.validate());
            println!();
        }
    }

    pub fn run(&mut self) {
        self.validate_args();

        match self.args[1].as_ref() {
            "add_block" => match self.args[2].as_ref() {
                "-data" => self.add_block(&self.args[3][..]),
                _ => panic!("invalid argument to command")
            },
            "print_chain" => self.print_chain(),
            _ => panic!("invalid command"),
        }
    }
}
