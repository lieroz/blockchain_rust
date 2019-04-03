use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::{TXOutput, TXOutputs};

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use typedb::{value, KV};

value!(
    enum StoreValue {
        String(String),
        TXOutputs(Vec<u8>),
    }
);

pub struct UTXOSet {
    store: KV<String, StoreValue>,
}

impl UTXOSet {
    pub fn get_store_name(node_id: &str) -> String {
        format!("utxo_set_{}.db", node_id)
    }

    pub fn new(store_name: &'static str) -> UTXOSet {
        UTXOSet {
            store: KV::<String, StoreValue>::new(store_name)
                .expect("error opening utxo set store"),
        }
    }

    pub fn find_spendable_outputs(
        &mut self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;

        for key in self.store.keys().expect("error getting keys from store") {
            let outs = match self
                .store
                .get(&key)
                .expect("error getting TXOutputs from store")
            {
                Some(o) => match o {
                    StoreValue::TXOutputs(outputs) => TXOutputs::deserialize(outputs),
                    _ => panic!("wrong type returned from store, StoreValue::TXOutputs expected"),
                },
                None => panic!("error getting TXOutputs from store"),
            };

            for (idx, out) in outs.outputs.iter().enumerate() {
                if out.is_locked_with_key(pub_key_hash) && accumulated < amount {
                    accumulated += out.value();
                    unspent_outputs
                        .entry(key.clone())
                        .or_insert(Vec::new())
                        .push(idx as i32);
                }
            }
        }

        (accumulated, unspent_outputs)
    }

    pub fn find_utxo(&mut self, pub_key_hash: &[u8]) -> Vec<TXOutput> {
        let mut utxos = Vec::new();

        for key in self.store.keys().expect("error getting keys from store") {
            let outs = match self
                .store
                .get(&key)
                .expect("error getting TXOutputs from store")
            {
                Some(o) => match o {
                    StoreValue::TXOutputs(outputs) => TXOutputs::deserialize(outputs),
                    _ => panic!("wrong type returned from store, StoreValue::TXOutputs expected"),
                },
                None => panic!("error getting TXOutputs from store"),
            };

            for out in outs.outputs {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.push(out);
                }
            }
        }

        utxos
    }

    pub fn count_transactions(&mut self) -> usize {
        self.store.keys().unwrap().len()
    }

    pub fn reindex(&mut self, node_id: &str, bc: &mut Blockchain) {
        let db_file = format!("utxo_set_{}.db", node_id);
        if Path::new(&db_file).exists() {
            let _ = fs::remove_file(&db_file);
        }

        self.store =
            KV::<String, StoreValue>::new(&db_file).expect("error opening utxo set store");
        let utxo = bc.find_utxo();

        for (tx_id, outs) in utxo {
            match self
                .store
                .insert(tx_id, StoreValue::TXOutputs(outs.serialize()))
            {
                Ok(_) => (),
                Err(err) => panic!("error while putting TXOutputs data into store: {}", err),
            };
        }
    }

    pub fn update(&mut self, block: &Block) {
        for tx in block.transactions() {
            if !tx.is_coinbase() {
                for tx_in in tx.v_in() {
                    let outs = match self
                        .store
                        .get(&tx_in.tx_id().to_string())
                        .expect("error getting TXOutputs from store")
                    {
                        Some(o) => match o {
                            StoreValue::TXOutputs(outputs) => TXOutputs::deserialize(outputs),
                            _ => panic!(
                                "wrong type returned from store, StoreValue::TXOutputs expected"
                            ),
                        },
                        None => panic!("error getting TXOutputs from store"),
                    };
                    let mut updated_outs = TXOutputs::new(Vec::new());

                    for (idx, out) in outs.outputs.iter().enumerate() {
                        if idx as i32 != tx_in.v_out() {
                            updated_outs.outputs.push(out.clone());
                        }
                    }

                    if updated_outs.outputs.is_empty() {
                        match self.store.remove(&tx_in.tx_id().to_string()) {
                            Ok(_) => (),
                            Err(err) => {
                                panic!("error while removing TXOutputs data from store: {}", err)
                            }
                        };
                    } else {
                        match self.store.insert(
                            tx_in.tx_id().to_string(),
                            StoreValue::TXOutputs(updated_outs.serialize()),
                        ) {
                            Ok(_) => (),
                            Err(err) => {
                                panic!("error while putting TXOutputs data into store: {}", err)
                            }
                        };
                    }
                }
            }

            let mut new_outputs = TXOutputs::new(Vec::new());

            for out in tx.v_out() {
                new_outputs.outputs.push(out.clone());
            }

            match self.store.insert(
                tx.id().to_string(),
                StoreValue::TXOutputs(new_outputs.serialize()),
            ) {
                Ok(_) => (),
                Err(err) => panic!("error while putting TXOutputs data into store: {}", err),
            };
        }
    }
}
