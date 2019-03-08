use crate::block::Block;

use std::cmp::Ordering;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

const MAX_NONCE: u64 = std::u64::MAX;
const TARGET_BYTE: i32 = 61;

#[derive(Debug)]
pub struct ProofOfWork<'a> {
    block: &'a Block,
    target: String,
}

impl<'a> ProofOfWork<'a> {
    fn generate_pof_border() -> String {
        let mut target = String::new();
        for i in (0..64).rev() {
            if i == TARGET_BYTE {
                target.push('1');
            } else {
                target.push('0');
            }
        }
        target
    }

    pub fn new(block: &Block) -> ProofOfWork {
        ProofOfWork{
            block,
            target: ProofOfWork::generate_pof_border(),
        }
    }

    fn prepare_data(&self, nonce: u64) -> String {
        format!("{}{}{}{}{}",
                self.block.prev_block_hash(),
                self.block.data(),
                self.block.timestamp(),
                self.target,
                nonce)
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
                Ordering::Greater => nonce += 1,
                _ => break,
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

