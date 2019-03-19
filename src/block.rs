use crate::proofofwork::ProofOfWork;
use crate::transaction::Transaction;

use std::time::SystemTime;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    timestamp: u64,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_block_hash: &str) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        let mut block = Block{
            timestamp: timestamp.as_secs(),
            transactions,
            prev_block_hash: prev_block_hash.to_string(),
            hash: String::new(),
            nonce: 0,
        };
        let pow = ProofOfWork::new(&block);
        let (nonce, hash) = pow.run();
        block.hash = hash;
        block.nonce = nonce;
        block
    }

    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new(vec![coinbase], "")
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("error serializing block")
    }

    pub fn deserialize(bytes: Vec<u8>) -> Block {
        bincode::deserialize(&bytes[..]).expect("error decerializing block")
    }

    pub fn hash_transactions(&self) -> String {
        let mut data = String::new();

        for tx in &self.transactions {
            data.push_str(tx.id());
        }

        let mut hasher = Sha256::new();
        hasher.input_str(&data[..]);
        hasher.result_str()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn prev_block_hash(&self) -> &str {
        &self.prev_block_hash[..]
    }

    pub fn hash(&self) -> &str {
        &self.hash[..]
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}
