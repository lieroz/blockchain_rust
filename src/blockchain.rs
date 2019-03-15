use crate::block::Block;

use typedb::{KV, value};

value!(
enum StoreValue {
    String(String),
    Block(Vec<u8>),
});

const TIP_KEY: &str = "l";
const CHECK_KEY: &str = "check_key";
const CHECK_VALUE: &str = "check_value";

pub struct Blockchain {
    store: KV<String, StoreValue>,
    tip: String,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut store = KV::<String, StoreValue>::new("./db.cab")
            .expect("error opening store");
        let tip;
        match store.get(&CHECK_KEY.to_string())
            .expect("error while extracting check data from store") {
            Some(_) => match store.get(&TIP_KEY.to_string())
                .expect("error while extracting tip data from store") {
                Some(o) => match o {
                    StoreValue::String(s) => tip = s,
                    _ => panic!("wrong type returned from store, storevalue::string was expected"),
                },
                None => panic!("tip data in store was corrupted"),
            },
            None => {
                match store.insert(CHECK_KEY.to_string(), StoreValue::String(CHECK_VALUE.to_string())) {
                    Ok(_) => (),
                    Err(err) => panic!("error while putting check data into store: {}", err),
                };
                let genesis = Block::new_genesis_block();
                match store.insert(genesis.hash().to_string(), StoreValue::Block(genesis.serialize())) {
                    Ok(_) => (),
                    Err(err) => panic!("error while putting block data into store: {}", err),
                };
                match store.insert(TIP_KEY.to_string(), StoreValue::String(genesis.hash().to_string())) {
                    Ok(_) => (),
                    Err(err) => panic!("error while putting tip data into store: {}", err),
                };
                tip = genesis.hash().to_string();
            },
        };
        Blockchain{store, tip}
    }

    pub fn add_block(&mut self, data: &str) {
        let new_block = Block::new(data, &self.tip[..]);
        match self.store.insert(new_block.hash().to_string(), StoreValue::Block(new_block.serialize())) {
            Ok(_) => (),
            Err(err) => panic!("error while putting new block into store {}", err),
        }
        match self.store.insert(TIP_KEY.to_string(), StoreValue::String(new_block.hash().to_string())) {
            Ok(_) => (),
            Err(err) => panic!("error while putting tip data into store {}", err),
        };
        self.tip = new_block.hash().to_string();
    }

    pub fn iter<'a>(&'a mut self) -> BlockchainIterator<'a> {
        BlockchainIterator{
            store: &mut self.store,
            tip: self.tip.clone(),
        }
    }
}

pub struct BlockchainIterator<'a> {
    store: &'a mut KV<String, StoreValue>,
    tip: String,
}

impl<'a> Iterator for BlockchainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        match self.store.get(&self.tip)
            .expect("error while extracting block from store") {
            Some(o) => match o {
                StoreValue::Block(block) => {
                    let block = Block::deserialize(block);
                    self.tip = block.prev_block_hash().to_string();
                    Some(block)
                },
                _ => panic!("wrong type returned from store, StoreValue::Block expected"),
            },
            None => None,
        }
    }
}
