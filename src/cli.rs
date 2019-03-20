use crate::blockchain::Blockchain;
use crate::proofofwork::ProofOfWork;
use crate::transaction::Transaction;

use std::process;

pub struct CLI<'a> {
    args: &'a [String],
}

impl<'a> CLI<'a> {
    pub fn new(args: &'a [String]) -> CLI<'a> {
        CLI { args }
    }

    fn print_usage(&self) {
        println!("Usage:");
        println!("    get_balance -address ADDRESS - get balance of ADDRESS");
        println!("    create_blockchain -address ADDRESS - create blockchain and send genesis block reward to ADDRESS");
        println!("    print_chain - print all the blocks of the blockchain");
        println!("    send -from FROM -to TO -amount AMOUNT - send AMOUNT of coins from FROM address to TO");
    }

    fn validate_args(&self) {
        if self.args.len() < 2 {
            self.print_usage();
            process::exit(1);
        }
    }

    fn print_chain(&self) {
        let mut bc = Blockchain::new();
        for block in bc.iter() {
            println!("Prev. hash: {}", block.prev_block_hash());
            println!("Hash: {}", block.hash());
            let pow = ProofOfWork::new(&block);
            println!("PoW: {}", pow.validate());
            println!();
        }
    }

    fn get_balance(&self, address: &str) {
        let mut bc = Blockchain::new();
        let mut balance = 0;
        let utxos = bc.find_utxo(address);

        for out in utxos {
            balance += out.value();
        }

        println!("Balance of {}: {}", address, balance);
    }

    fn create(&self, address: &str) {
        let _ = Blockchain::create(address);
        println!("Success!");
    }

    fn send(&self, from: &str, to: &str, amount: i32) {
        let mut bc = Blockchain::new();
        let tx = Transaction::new_utxo_tx(from, to, amount, &mut bc);
        bc.mine_block(vec![tx]);
        println!("Success!");
    }

    pub fn run(&self) {
        self.validate_args();

        match self.args[1].as_ref() {
            "get_balance" => match self.args[2].as_ref() {
                "-address" => self.get_balance(&self.args[3][..]),
                _ => panic!("invalid argument to command"),
            },
            "send" => match self.args[2].as_ref() {
                "-from" => match self.args[4].as_ref() {
                    "-to" => match self.args[6].as_ref() {
                        "-amount" => self.send(
                            &self.args[3][..],
                            &self.args[5][..],
                            self.args[7].parse::<i32>().unwrap(),
                        ),
                        _ => self.print_usage(),
                    },
                    _ => self.print_usage(),
                },
                _ => self.print_usage(),
            },
            "create_blockchain" => match self.args[2].as_ref() {
                "-address" => self.create(&self.args[3]),
                _ => self.print_usage(),
            },
            "print_chain" => self.print_chain(),
            _ => self.print_usage(),
        }
    }
}
