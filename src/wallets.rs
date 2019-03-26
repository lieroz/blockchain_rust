use crate::wallet::Wallet;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

const WALLET_FILE: &str = "wallet.dat";

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Wallets {
        Wallets {
            wallets: HashMap::new(),
        }
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        address
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }
        addresses
    }

    pub fn get_wallet(&self, address: &str) -> &Wallet {
        &self.wallets[address]
    }

    pub fn load_from_file(&mut self) {
        if !Path::new(WALLET_FILE).exists() {
            panic!("can't load wallets from file that doesn't exists");
        }

        let data = fs::read(WALLET_FILE).expect("error happened when reading from wallet file");
        self.wallets = bincode::deserialize(&data).expect("error decerializing wallets from file");
    }

    pub fn save_to_file(&self) {
        let data = bincode::serialize(&self).expect("error serializing wallets");
        fs::write(WALLET_FILE, &data).expect("error writing serialized wallets to file");
    }
}
