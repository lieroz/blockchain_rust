use crate::proofofwork::ProofOfWork;

use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    timestamp: u64,
    data: String,
    prev_block_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    pub fn new(data: &str, prev_block_hash: &str) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        let mut block = Block{
            timestamp: timestamp.as_secs(),
            data: data.to_string(),
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

    pub fn new_genesis_block() -> Block {
        Block::new("Genesis Block", "")
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("error serializing block")
    }

    pub fn deserialize(bytes: Vec<u8>) -> Block {
        bincode::deserialize(&bytes[..]).expect("error decerializing block")
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn data(&self) -> &str {
        &self.data[..]
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
