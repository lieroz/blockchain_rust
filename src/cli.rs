use crate::blockchain::Blockchain;
use crate::proofofwork::ProofOfWork;
use crate::transaction::Transaction;
use crate::wallets::Wallets;
use crate::wallet::Wallet;

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
        println!("    createblockchain -address ADDRESS - create blockchain and send genesis block reward to ADDRESS");
        println!("    createwallet - generates a new key pair abd saves it into the wallet file");
        println!("    getbalance -address ADDRESS - get balance of ADDRESS");
        println!("    listaddresses - lists all addresses from the wallet file");
        println!("    printchain - print all the blocks of the blockchain");
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
            println!("============ Block {} ============", block.hash());
            println!("Prev. block: {}", block.prev_block_hash());
            let pow = ProofOfWork::new(&block);
            println!("PoW: {}\n", pow.validate());

            for tx in block.transactions() {
                println!("{}", tx.to_string());
            }

            println!();
        }
    }

    fn get_balance(&self, address: &str) {
        if !Wallet::validate_address(address) {
            panic!("ERROR: Address is not valid");
        }

        let mut bc = Blockchain::new();

        let mut balance = 0;
        let pub_key_hash = bs58::decode(address)
            .into_vec()
            .expect("error decoding address using base 58");
        let pub_key_hash = pub_key_hash[1..pub_key_hash.len()-4].to_vec();
        let utxos = bc.find_utxo(&pub_key_hash[..]);

        for out in utxos {
            balance += out.value();
        }

        println!("Balance of {}: {}", address, balance);
    }

    fn create_blockchain(&self, address: &str) {
        if !Wallet::validate_address(address) {
            panic!("ERROR: Address is not valid");
        }
        let _ = Blockchain::create(address);
        println!("Success!");
    }

    fn send(&self, from: &str, to: &str, amount: i32) {
        if !Wallet::validate_address(from) {
            panic!("ERROR: Sender address is not valid");
        }

        if !Wallet::validate_address(to) {
            panic!("ERROR: Recipient address is not valid");
        }

        let mut bc = Blockchain::new();
        let tx = Transaction::new_utxo_tx(from, to, amount, &mut bc);
        bc.mine_block(vec![tx]);
        println!("Success!");
    }

    fn create_wallet(&self) {
        let mut wallets = Wallets::new();
        let address = wallets.create_wallet();
        println!("Your new address: {}", address);
    }

    fn list_addresses(&self) {
        let mut wallets = Wallets::new();
        let addresses = wallets.get_addresses();

        for address in addresses {
            println!("{}", address);
        }
    }

    pub fn run(&self) {
        self.validate_args();

        match self.args[1].as_ref() {
            "getbalance" => match self.args[2].as_ref() {
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
            "createblockchain" => match self.args[2].as_ref() {
                "-address" => self.create_blockchain(&self.args[3]),
                _ => self.print_usage(),
            },
            "createwallet" => self.create_wallet(),
            "listaddresses" => self.list_addresses(),
            "printchain" => self.print_chain(),
            _ => self.print_usage(),
        }
    }
}
