use crate::blockchain::Blockchain;
use crate::wallet::{self, Wallet};
use crate::wallets::Wallets;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

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
            data = format!("Reward to: {}", to);
        }

        let mut tx = Transaction {
            id: String::new(),
            v_in: vec![TXInput::new("", -1, Vec::new(), data.as_bytes().to_vec())],
            v_out: vec![TXOutput::new(SUBSIDY, to)],
        };

        tx.set_id();
        tx
    }

    pub fn new_utxo_tx(from: &str, to: &str, amount: i32, bc: &mut Blockchain) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let wallet = Wallets::new().get_wallet(from);
        let (acc, valid_outputs) = bc.find_spendable_outputs(from, amount);

        if acc < amount {
            panic!("ERROR: Not enough funds")
        }

        for (idx, outs) in valid_outputs.iter() {
            for out in outs {
                let input = TXInput::new(idx, *out, Vec::new(), wallet.public_key());
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
        tx
    }

    pub fn is_coinbase(&self) -> bool {
        self.v_in.len() == 1 && self.v_in[0].tx_id.len() == 0 && self.v_in[0].v_out == -1
    }

    pub fn sign(&self, priv_key: ) {
    }

    pub fn trimmed_copy(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for tx_in in self.v_in {
            inputs.push(TXInput::new(tx_in.tx_id(), tx_in.v_out(), Vec::new(), Vec::new()));
        }

        for tx_out in self.v_out {
            outputs.push(tx_out.clone());
        }

        Transaction{
            id: self.id,
            v_in: inputs,
            v_out: outputs,
        }
    }

    pub fn verify(&self) {
    }

    fn set_id(&mut self) {
        let mut hasher = Sha256::new();
        let data = bincode::serialize(&self).expect("error serializing transaction");
        hasher.input(&data);
        self.id = hasher.result_str()
    }

    pub fn id(&self) -> &str {
        &self.id[..]
    }

    pub fn v_in(&self) -> &[TXInput] {
        &self.v_in
    }

    pub fn v_out(&self) -> &[TXOutput] {
        &self.v_out
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        let mut lines = String::from(format!("--- Transaction {}:", self.id));

        for (i, input) in self.v_in.iter().enumerate() {
            lines.push_str(&format!("     Input {}:", i)[..]);
            lines.push_str(&format!("       TXID:      {}", input.tx_id())[..]);
            lines.push_str(&format!("       Out:       {}", input.v_out())[..]);
            lines.push_str(&format!("       Signature: {}", input.signature())[..]);
            lines.push_str(&format!("       PubKey:    {}", input.pub_key())[..]);
        }

        for (i, output) in self.v_out.iter().enumerate() {
            lines.push_str(&format!("     Output {}:", i)[..]);
            lines.push_str(&format!("       Value:  {}", output.value())[..]);
            lines.push_str(&format!("       Script: {}", output.pub_key_hash())[..]);
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

    pub fn uses_key(&self, pub_key_hash: &str) -> bool {
        let pub_key_hash_bytes = pub_key_hash.as_bytes();
        let locking_hash = Wallet::hash_pub_key(&pub_key_hash_bytes);
        locking_hash.len() == pub_key_hash_bytes.len()
            && locking_hash
                .iter()
                .zip(pub_key_hash_bytes)
                .all(|(a, b)| a == b)
    }

    pub fn tx_id(&self) -> &str {
        &self.tx_id[..]
    }

    pub fn v_out(&self) -> i32 {
        self.v_out
    }

    pub fn signature(&self) -> String {
        std::str::from_utf8(&self.signature).unwrap().to_string()
    }

    pub fn pub_key(&self) -> String {
        std::str::from_utf8(&self.public_key).unwrap().to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXOutput {
    value: i32,
    pub_key_hash: Vec<u8>,
}

impl TXOutput {
    pub fn new(amount: i32, to: &str) -> TXOutput {
        let mut txo = TXOutput {
            value: amount,
            pub_key_hash: Vec::new(),
        };
        txo.lock(to);
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

    pub fn pub_key_hash(&self) -> String {
        std::str::from_utf8(&self.pub_key_hash).unwrap().to_string()
    }

}
