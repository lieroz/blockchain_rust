use crate::block::Block;
use crate::transaction::{TXOutputs, Transaction};

use std::collections::HashMap;
use typedb::{value, KV};

value!(
    enum StoreValue {
        String(String),
        Block(Vec<u8>),
    }
);

const TIP_KEY: &str = "l";
const CHECK_KEY: &str = "check_key";
const CHECK_VALUE: &str = "check_value";
const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

pub struct Blockchain {
    store: KV<String, StoreValue>,
    tip: String,
}

impl Blockchain {
    fn exists(store: &mut KV<String, StoreValue>) -> bool {
        match store
            .get(&CHECK_KEY.to_string())
            .expect("error while extracting check data from store")
        {
            Some(_) => true,
            None => false,
        }
    }

    pub fn new(node_id: &str) -> Blockchain {
        let db_file = Box::leak(Box::new(format!("blockchain_{}.db", node_id)));
        let mut store =
            KV::<String, StoreValue>::new(db_file).expect("error opening blockchain store");

        if !Blockchain::exists(&mut store) {
            panic!("no existsing blockchain found")
        }

        let tip = match store
            .get(&TIP_KEY.to_string())
            .expect("error while extracting tip data from store")
        {
            Some(o) => match o {
                StoreValue::String(s) => s,
                _ => panic!("wrong type returned from store, storevalue::string was expected"),
            },
            None => panic!("tip data in store was corrupted"),
        };

        Blockchain { store, tip }
    }

    pub fn create(node_id: &str, address: &str) -> Blockchain {
        let db_file = Box::leak(Box::new(format!("blockchain_{}.db", node_id)));
        let mut store = KV::<String, StoreValue>::new(db_file).expect("error opening store");

        if Blockchain::exists(&mut store) {
            panic!("blockchain already exists")
        }

        match store.insert(
            CHECK_KEY.to_string(),
            StoreValue::String(CHECK_VALUE.to_string()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting check data into store: {}", err),
        };
        let cbtx = Transaction::new_coin_base_tx(address, GENESIS_COINBASE_DATA);
        let genesis = Block::new_genesis_block(cbtx);
        match store.insert(
            genesis.hash().to_string(),
            StoreValue::Block(genesis.serialize()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting block data into store: {}", err),
        };
        match store.insert(
            TIP_KEY.to_string(),
            StoreValue::String(genesis.hash().to_string()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting tip data into store: {}", err),
        };

        let tip = genesis.hash().to_string();
        Blockchain { store, tip }
    }

    pub fn add_block(&mut self, block: &Block) {
        match self.store.insert(
            block.hash().to_string(),
            StoreValue::Block(block.serialize()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting new block into store {}", err),
        }
        match self.store.insert(
            TIP_KEY.to_string(),
            StoreValue::String(block.hash().to_string()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting tip data into store {}", err),
        };
        self.tip = block.hash().to_string();
    }

    pub fn get_best_height(&mut self) -> i32 {
        match self
            .store
            .get(&self.tip)
            .expect("error while extracting Block from store")
        {
            Some(o) => match o {
                StoreValue::Block(bytes) => Block::deserialize(bytes).height(),
                _ => panic!("wrong type returned from store, StoreValue::Block was expected"),
            },
            None => panic!("error, tip was corrupted"),
        }
    }

    pub fn get_block(&mut self, block_hash: &str) -> Block {
        match self
            .store
            .get(&block_hash.to_string())
            .expect("error while extracting Block from store")
        {
            Some(o) => match o {
                StoreValue::Block(bytes) => Block::deserialize(bytes),
                _ => panic!("wrong type returned from store, StoreValue::Block was expected"),
            },
            None => panic!("error, block was not found"),
        }
    }

    pub fn get_block_hashes(&mut self) -> Vec<String> {
        let mut block_hashes = Vec::new();

        for block in self.iter() {
            block_hashes.push(block.hash().to_string());
        }

        block_hashes
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Block {
        for tx in &transactions {
            if !self.verify_transaction(tx) {
                panic!("ERROR: Invalid transaction");
            }
        }

        let height = self.get_best_height();
        let new_block = Block::new(transactions, &self.tip[..], height + 1);
        match self.store.insert(
            new_block.hash().to_string(),
            StoreValue::Block(new_block.serialize()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting new block into store {}", err),
        }
        match self.store.insert(
            TIP_KEY.to_string(),
            StoreValue::String(new_block.hash().to_string()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting tip data into store {}", err),
        };
        self.tip = new_block.hash().to_string();
        new_block
    }

    pub fn iter<'a>(&'a mut self) -> BlockchainIterator<'a> {
        BlockchainIterator {
            store: &mut self.store,
            tip: self.tip.clone(),
        }
    }

    pub fn find_utxo(&mut self) -> HashMap<String, TXOutputs> {
        let mut utxo: HashMap<String, TXOutputs> = HashMap::new();
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.transactions() {
                'outputs: for (idx, out_tx) in tx.v_out().iter().enumerate() {
                    if spent_txos.contains_key(tx.id()) {
                        for spent_out in spent_txos[tx.id()].iter() {
                            if *spent_out == idx as i32 {
                                continue 'outputs;
                            }
                        }
                    }

                    let outs = utxo
                        .entry(tx.id().to_string())
                        .or_insert(TXOutputs::new(Vec::new()));
                    outs.outputs.push(out_tx.clone());
                    let tmp = outs.clone();
                    utxo.insert(tx.id().to_string(), tmp);
                }

                if !tx.is_coinbase() {
                    for in_tx in tx.v_in() {
                        spent_txos
                            .entry(in_tx.tx_id().to_string())
                            .or_insert(Vec::new())
                            .push(in_tx.v_out());
                    }
                }
            }
        }

        utxo
    }

    pub fn find_transaction(&mut self, id: &str) -> Transaction {
        for block in self.iter() {
            for tx in block.transactions() {
                if tx.id() == id {
                    return tx.clone();
                }
            }
        }

        panic!("Transaction is not found");
    }

    pub fn sign_transaction(&mut self, tx: &mut Transaction, pkcs8_bytes: &[u8]) {
        let mut prev_txs = HashMap::new();

        for tx_in in tx.v_in() {
            let prev_tx = self.find_transaction(tx_in.tx_id());
            prev_txs.insert(prev_tx.id().to_string(), prev_tx);
        }

        tx.sign(pkcs8_bytes, &prev_txs);
    }

    pub fn verify_transaction(&mut self, tx: &Transaction) -> bool {
        if tx.is_coinbase() {
            return true;
        }

        let mut prev_txs = HashMap::new();

        for tx_in in tx.v_in() {
            let prev_tx = self.find_transaction(tx_in.tx_id());
            prev_txs.insert(prev_tx.id().to_string(), prev_tx);
        }

        tx.verify(&prev_txs)
    }
}

pub struct BlockchainIterator<'a> {
    store: &'a mut KV<String, StoreValue>,
    tip: String,
}

impl<'a> Iterator for BlockchainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        match self
            .store
            .get(&self.tip)
            .expect("error while extracting block from store")
        {
            Some(o) => match o {
                StoreValue::Block(block) => {
                    let block = Block::deserialize(block);
                    self.tip = block.prev_block_hash().to_string();
                    Some(block)
                }
                _ => panic!("wrong type returned from store, StoreValue::Block expected"),
            },
            None => None,
        }
    }
}
