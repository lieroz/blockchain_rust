use crate::proofofwork::ProofOfWork;

use std::time::SystemTime;

#[derive(Debug)]
pub struct Block {
    timestamp: u64,
    data: String,
    prev_block_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    pub fn new(data: &str, prev_block_hash: &str) -> Block {
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let mut block = Block{
            timestamp,
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
