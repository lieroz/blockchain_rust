use crate::block::Block;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::cmp::Ordering;

const MAX_NONCE: u64 = std::u64::MAX;
const TARGET_BYTE: i32 = 61;

#[derive(Debug)]
pub struct ProofOfWork<'a> {
    block: &'a Block,
    target: String,
}

impl<'a> ProofOfWork<'a> {
    pub fn new(block: &Block) -> ProofOfWork {
        ProofOfWork {
            block,
            target: (0..64)
                .rev()
                .map(|x| if x == TARGET_BYTE { '1' } else { '0' })
                .collect(),
        }
    }

    fn prepare_data(&self, nonce: u64) -> String {
        format!(
            "{}{}{}{}{}",
            self.block.prev_block_hash(),
            self.block.hash_transactions(),
            self.block.timestamp(),
            self.target,
            nonce
        )
    }

    pub fn run(&self) -> (u64, String) {
        let mut nonce = 0;
        let mut hash = String::new();
        let mut hasher = Sha256::new();

        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);
            hasher.input_str(&data[..]);
            hash = hasher.result_str();

            match hash.cmp(&self.target) {
                Ordering::Less => break,
                _ => nonce += 1,
            }

            hasher.reset();
        }

        (nonce, hash)
    }

    pub fn validate(&self) -> bool {
        let data = self.prepare_data(self.block.nonce());
        let mut hasher = Sha256::new();
        hasher.input_str(&data[..]);
        let hash = hasher.result_str();
        match hash.cmp(&self.target) {
            Ordering::Less => true,
            _ => false,
        }
    }
}
