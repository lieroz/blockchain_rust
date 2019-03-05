use std::time::SystemTime;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Debug)]
pub struct Block {
    timestamp: u64,
    data: String,
    prev_block_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(data: &str, prev_block_hash: &str) -> Block {
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        let mut hasher = Sha256::new();
        hasher.input_str(&format!("{}{}{}", prev_block_hash, data, timestamp)[..]);

        Block{
            timestamp,
            data: data.to_string(),
            prev_block_hash: prev_block_hash.to_string(),
            hash: hasher.result_str(),
        }
    }

    pub fn new_genesis_block() -> Block {
        Block::new("Genesis Block", "")
    }
}
