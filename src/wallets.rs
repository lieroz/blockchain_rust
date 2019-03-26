use crate::wallet::Wallet;

use typedb::{value, KV};

value!(
    enum StoreValue {
        String(String),
        Wallet(Vec<u8>),
    }
);

const WALLET_FILE: &str = "wallet.db";

pub struct Wallets {
    store: KV<String, StoreValue>,
}

impl Wallets {
    pub fn new() -> Wallets {
        let store = KV::<String, StoreValue>::new(WALLET_FILE).expect("error opening wallet store");
        Wallets{ store }
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        match self.store.insert(
            address.clone(),
            StoreValue::Wallet(wallet.serialize()),
        ) {
            Ok(_) => (),
            Err(err) => panic!("error while putting wallet data into store: {}", err),
        };
        address
    }

    pub fn get_addresses(&mut self) -> Vec<String> {
        self.store.keys().expect("error getting keys from store")
    }

    pub fn get_wallet(&mut self, address: &str) -> Wallet {
        for key in self.store.keys().expect("error getting keys from store") {
            if key == address {
                return match self.store.get(&key)
                    .expect("error getting wallet from store") {
                    Some(o) => match o {
                        StoreValue::Wallet(wallet) => {
                            Wallet::deserialize(wallet)
                        }
                        _ => panic!("wrong type returned from store, StoreValue::Block expected"),
                    },
                    None => panic!("error getting wallet from store"),
                };
            }
        }

        panic!("no wallet with given address {} was found", address);
    }
}

