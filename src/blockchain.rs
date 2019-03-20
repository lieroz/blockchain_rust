use crate::block::Block;
use crate::transaction::{TXOutput, Transaction};

use std::collections::HashMap;
use typedb::{value, KV};

value!(
    enum StoreValue {
        String(String),
        Block(Vec<u8>),
    }
);

const STORE_FILE: &str = "db.cab";
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

    pub fn new() -> Blockchain {
        let mut store = KV::<String, StoreValue>::new(STORE_FILE).expect("error opening store");

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

    pub fn create(address: &str) -> Blockchain {
        let mut store = KV::<String, StoreValue>::new(STORE_FILE).expect("error opening store");

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

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        let new_block = Block::new(transactions, &self.tip[..]);
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
    }

    pub fn iter<'a>(&'a mut self) -> BlockchainIterator<'a> {
        BlockchainIterator {
            store: &mut self.store,
            tip: self.tip.clone(),
        }
    }

    pub fn find_unspent_transactions(&mut self, address: &str) -> Vec<Transaction> {
        let mut unspent_txs: Vec<Transaction> = Vec::new();
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

                    if out_tx.can_be_unlocked(address) {
                        unspent_txs.push(tx.clone());
                    }
                }

                if !tx.is_coinbase() {
                    for in_tx in tx.v_in() {
                        if in_tx.can_unlock_output(address) {
                            spent_txos
                                .entry(in_tx.tx_id().to_string())
                                .or_insert(Vec::new())
                                .push(in_tx.v_out());
                        }
                    }
                }
            }
        }

        unspent_txs
    }

    pub fn find_utxo(&mut self, address: &str) -> Vec<TXOutput> {
        let mut unspent_txos: Vec<TXOutput> = Vec::new();
        let unspent_txs = self.find_unspent_transactions(address);

        for tx in unspent_txs {
            for out in tx.v_out() {
                if out.can_be_unlocked(address) {
                    unspent_txos.push(out.clone());
                }
            }
        }

        unspent_txos
    }

    pub fn find_spendable_outputs(
        &mut self,
        address: &str,
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let unspent_txs = self.find_unspent_transactions(address);
        let mut accumulated = 0;

        'work: for tx in unspent_txs {
            for (idx, out) in tx.v_out().iter().enumerate() {
                if out.can_be_unlocked(address) && accumulated < amount {
                    accumulated += out.value();
                    unspent_outputs
                        .entry(tx.id().to_string())
                        .or_insert(Vec::new())
                        .push(idx as i32);

                    if accumulated >= amount {
                        break 'work;
                    }
                }
            }
        }

        (accumulated, unspent_outputs)
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
