use crate::blockchain::Blockchain;
use crate::proofofwork::ProofOfWork;
use crate::transaction::Transaction;
use crate::utxo_set::UTXOSet;
use crate::wallet::Wallet;
use crate::wallets::Wallets;

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
        println!("    reindexutxo - rebuilds the utxo set");
        println!("    send -from FROM -to TO -amount AMOUNT - send AMOUNT of coins from FROM address to TO;
                 mine on the same node, when -mine is set");
        println!("    startnode -miner ADDRESS - start a node with ID specified in NODE_ID env. var. -miner enables mining");
    }

    fn validate_args(&self) {
        if self.args.len() < 2 {
            self.print_usage();
            process::exit(1);
        }
    }

    fn create_blockchain(&self, node_id: &str, address: &str) {
        if !Wallet::validate_address(address) {
            panic!("ERROR: Address is not valid");
        }
        let mut bc = Blockchain::create(node_id, address);
        let mut utxo_set = UTXOSet::new(node_id);
        utxo_set.reindex(node_id, &mut bc);

        println!("Success!");
    }

    fn create_wallet(&self, node_id: &str) {
        let mut wallets = Wallets::new(node_id);
        let address = wallets.create_wallet();
        println!("Your new address: {}", address);
    }

    fn get_balance(&self, node_id: &str, address: &str) {
        if !Wallet::validate_address(address) {
            panic!("ERROR: Address is not valid");
        }

        let mut utxo_set = UTXOSet::new(node_id);

        let mut balance = 0;
        let pub_key_hash = bs58::decode(address)
            .into_vec()
            .expect("error decoding address using base 58");
        let pub_key_hash = pub_key_hash[1..pub_key_hash.len() - 4].to_vec();
        let utxos = utxo_set.find_utxo(&pub_key_hash[..]);

        for out in utxos {
            balance += out.value();
        }

        println!("Balance of {}: {}", address, balance);
    }

    fn list_addresses(&self, node_id: &str) {
        let mut wallets = Wallets::new(node_id);
        let addresses = wallets.get_addresses();

        for address in addresses {
            println!("{}", address);
        }
    }

    fn print_chain(&self, node_id: &str) {
        let mut bc = Blockchain::new(node_id);
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

    fn reindex_utxo(&self, node_id: &str) {
        let mut bc = Blockchain::new(node_id);
        let mut utxo_set = UTXOSet::new(node_id);
        utxo_set.reindex(node_id, &mut bc);

        let count = utxo_set.count_transactions();
        println!("Done! There are {} transactions in the UTXO set.", count);
    }

    fn send(&self, node_id: &str, from: &str, to: &str, amount: i32) {
        if !Wallet::validate_address(from) {
            panic!("ERROR: Sender address is not valid");
        }

        if !Wallet::validate_address(to) {
            panic!("ERROR: Recipient address is not valid");
        }

        let mut bc = Blockchain::new(node_id);
        let mut utxo_set = UTXOSet::new(node_id);
        let wallet = Wallets::new(node_id).get_wallet(from);
        let tx = Transaction::new_utxo_tx(&wallet, to, amount, &mut bc, &mut utxo_set);
        let cbtx = Transaction::new_coin_base_tx(from, "");
        let block = bc.mine_block(vec![cbtx, tx]);
        utxo_set.update(&block);
        println!("Success!");
    }

    fn start_node(&self, node_id: &str) {
    }

    pub fn run(&self) {
        self.validate_args();
        let node_id = std::env::var("NODE_ID")
            .expect("error reading NODE_ID from env");

        match self.args[1].as_ref() {
            "createblockchain" => match self.args[2].as_ref() {
                "-address" => self.create_blockchain(&node_id, &self.args[3]),
                _ => self.print_usage(),
            },
            "createwallet" => self.create_wallet(&node_id),
            "getbalance" => match self.args[2].as_ref() {
                "-address" => self.get_balance(&node_id, &self.args[3][..]),
                _ => panic!("invalid argument to command"),
            },
            "listaddresses" => self.list_addresses(&node_id),
            "printchain" => self.print_chain(&node_id),
            "reindexutxo" => self.reindex_utxo(&node_id),
            "send" => match self.args[2].as_ref() {
                "-from" => match self.args[4].as_ref() {
                    "-to" => match self.args[6].as_ref() {
                        "-amount" => self.send(
                            &node_id,
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
            "startnode" => self.start_node(&node_id),
            _ => self.print_usage(),
        }
    }
}
