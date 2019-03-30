use crate::blockchain::Blockchain;
use crate::utxo_set::UTXOSet;
use crate::wallet::{self, Wallet};
use crate::wallets::Wallets;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use ring::signature;
use std::collections::HashMap;

extern crate rand;

use rand::seq::SliceRandom;
use rand::thread_rng;

const SUBSIDY: i32 = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    id: String,
    v_in: Vec<TXInput>,
    v_out: Vec<TXOutput>,
}

impl Transaction {
    pub fn new_coin_base_tx(to: &str, data: &str) -> Transaction {
        let mut data = String::from(data);

        if data.is_empty() {
            const CHARSET: &[u8] =  b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
            abcdefghijklmnopqrstuvwxyz\
            0123456789)(*&^%$#@!~";

            let mut rng = thread_rng();
            let password: Option<String> = (0..30)
                .map(|_| Some(*CHARSET.choose(&mut rng)? as char))
                .collect();
            data = password.unwrap();
        }

        let mut tx = Transaction {
            id: String::new(),
            v_in: vec![TXInput::new("", -1, Vec::new(), data.as_bytes().to_vec())],
            v_out: vec![TXOutput::new(SUBSIDY, to)],
        };

        tx.set_id();
        tx
    }

    pub fn new_utxo_tx(
        from: &str,
        to: &str,
        amount: i32,
        bc: &mut Blockchain,
        utxo_set: &mut UTXOSet,
    ) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut wallets = Wallets::new();
        let wallet = wallets.get_wallet(from);
        let pub_key_hash = Wallet::hash_pub_key(wallet.public_key());
        let (acc, valid_outputs) = utxo_set.find_spendable_outputs(&pub_key_hash[..], amount);

        if acc < amount {
            panic!("ERROR: Not enough funds")
        }

        for (idx, outs) in valid_outputs.iter() {
            for out in outs {
                let input = TXInput::new(idx, *out, Vec::new(), wallet.public_key().to_vec());
                inputs.push(input);
            }
        }

        outputs.push(TXOutput::new(amount, to));

        if acc > amount {
            outputs.push(TXOutput::new(acc - amount, from))
        }

        let mut tx = Transaction {
            id: String::new(),
            v_in: inputs,
            v_out: outputs,
        };

        tx.set_id();
        bc.sign_transaction(&mut tx, wallet.pkcs8_bytes());
        tx
    }

    pub fn is_coinbase(&self) -> bool {
        self.v_in.len() == 1 && self.v_in[0].tx_id.len() == 0 && self.v_in[0].v_out == -1
    }

    pub fn sign(&mut self, pkcs8_bytes: &[u8], prev_txs: &HashMap<String, Transaction>) {
        if self.is_coinbase() {
            return;
        }

        for tx_in in self.v_in() {
            if prev_txs[tx_in.tx_id()].id().is_empty() {
                panic!("error, previous transaction is not correct");
            }
        }

        let mut tx_copy = self.trimmed_copy();
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(pkcs8_bytes))
            .expect("error converting bytes to key pair");
        let mut signatures = Vec::new();

        for (i, tx_in) in self.v_in().iter().enumerate() {
            let prev_tx = &prev_txs[tx_in.tx_id()];
            let tx_copy_in = &mut tx_copy.mut_v_in()[i];
            tx_copy_in.set_pub_key(prev_tx.v_out()[tx_in.v_out() as usize].pub_key_hash());
            tx_copy.set_id();
            let sig = key_pair.sign(tx_copy.id().as_bytes());
            signatures.push(sig);
        }

        for (i, sig) in signatures.iter().enumerate() {
            self.v_in[i].set_signature(sig.as_ref());
        }
    }

    pub fn trimmed_copy(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for tx_in in self.v_in() {
            inputs.push(TXInput::new(
                tx_in.tx_id(),
                tx_in.v_out(),
                Vec::new(),
                Vec::new(),
            ));
        }

        for tx_out in self.v_out() {
            outputs.push(tx_out.clone());
        }

        Transaction {
            id: self.id.clone(),
            v_in: inputs,
            v_out: outputs,
        }
    }

    pub fn verify(&self, prev_txs: &HashMap<String, Transaction>) -> bool {
        if self.is_coinbase() {
            return true;
        }

        for tx_in in self.v_in() {
            if prev_txs[tx_in.tx_id()].id().is_empty() {
                panic!("error, previous transaction is not correct");
            }
        }

        let mut tx_copy = self.trimmed_copy();

        for (i, tx_in) in self.v_in().iter().enumerate() {
            let prev_tx = &prev_txs[tx_in.tx_id()];
            let tx_copy_in = &mut tx_copy.mut_v_in()[i];
            tx_copy_in.set_pub_key(prev_tx.v_out()[tx_in.v_out() as usize].pub_key_hash());
            tx_copy.set_id();

            match signature::verify(
                &signature::ED25519,
                untrusted::Input::from(tx_in.pub_key()),
                untrusted::Input::from(tx_copy.id().as_bytes()),
                untrusted::Input::from(tx_in.signature()),
            ) {
                Err(_) => return false,
                _ => continue,
            }
        }

        true
    }

    fn set_id(&mut self) {
        let mut hasher = Sha256::new();
        let data = bincode::serialize(&self).expect("error serializing transaction");
        hasher.input(&data);
        self.id = hasher.result_str()
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("error serializing Transaction")
    }

    pub fn id(&self) -> &str {
        &self.id[..]
    }

    pub fn v_in(&self) -> &[TXInput] {
        &self.v_in
    }

    pub fn mut_v_in(&mut self) -> &mut [TXInput] {
        &mut self.v_in
    }

    pub fn v_out(&self) -> &[TXOutput] {
        &self.v_out
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        let mut lines = String::from(format!("--- Transaction {}:\n", self.id));

        for (i, input) in self.v_in().iter().enumerate() {
            lines.push_str(&format!("     Input {}:\n", i)[..]);
            lines.push_str(&format!("       TXID:      {}\n", input.tx_id())[..]);
            lines.push_str(&format!("       Out:       {}\n", input.v_out())[..]);
            lines.push_str(&format!("       Signature: {:?}\n", input.signature())[..]);
            lines.push_str(&format!("       PubKey:    {:?}\n\n", input.pub_key())[..]);
        }

        for (i, output) in self.v_out().iter().enumerate() {
            lines.push_str(&format!("     Output {}:\n", i)[..]);
            lines.push_str(&format!("       Value:  {}\n", output.value())[..]);
            lines.push_str(&format!("       Script: {:?}\n\n", output.pub_key_hash())[..]);
        }

        lines
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXInput {
    tx_id: String,
    v_out: i32,
    signature: Vec<u8>,
    public_key: Vec<u8>,
}

impl TXInput {
    pub fn new(tx_id: &str, v_out: i32, signature: Vec<u8>, public_key: Vec<u8>) -> TXInput {
        TXInput {
            tx_id: tx_id.to_string(),
            v_out,
            signature,
            public_key,
        }
    }

    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = Wallet::hash_pub_key(&self.public_key);
        locking_hash.len() == pub_key_hash.len()
            && locking_hash.iter().zip(pub_key_hash).all(|(a, b)| a == b)
    }

    pub fn tx_id(&self) -> &str {
        &self.tx_id[..]
    }

    pub fn v_out(&self) -> i32 {
        self.v_out
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature[..]
    }

    pub fn set_signature(&mut self, signature: &[u8]) {
        self.signature = signature.to_vec()
    }

    pub fn pub_key(&self) -> &[u8] {
        &self.public_key[..]
    }

    pub fn set_pub_key(&mut self, bytes: &[u8]) {
        self.public_key = bytes.to_vec();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXOutput {
    value: i32,
    pub_key_hash: Vec<u8>,
}

impl TXOutput {
    pub fn new(amount: i32, address: &str) -> TXOutput {
        let mut txo = TXOutput {
            value: amount,
            pub_key_hash: Vec::new(),
        };
        txo.lock(address);
        txo
    }

    fn lock(&mut self, address: &str) {
        let pub_key_hash = bs58::decode(address)
            .into_vec()
            .expect("error decoding address using base 58");
        let size = pub_key_hash.len() - wallet::ADDRESS_CHECKSUM_LEN;
        let pub_key_hash = pub_key_hash[1..size].to_vec();
        self.pub_key_hash = pub_key_hash;
    }

    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.len() == pub_key_hash.len()
            && self
                .pub_key_hash
                .iter()
                .zip(pub_key_hash)
                .all(|(a, b)| a == b)
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn pub_key_hash(&self) -> &[u8] {
        &self.pub_key_hash
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}

impl TXOutputs {
    pub fn new(outputs: Vec<TXOutput>) -> TXOutputs {
        TXOutputs { outputs }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("error serializing TXOutputs")
    }

    pub fn deserialize(bytes: Vec<u8>) -> TXOutputs {
        bincode::deserialize(&bytes[..]).expect("error decerializing TXOutputs")
    }
}
