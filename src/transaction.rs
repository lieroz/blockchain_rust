use crate::blockchain::Blockchain;

use crypto::sha2::Sha256;
use crypto::digest::Digest;

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
        let mut tx = Transaction{
            id: String::new(),
            v_in: vec![TXInput::new("", -1, &data[..])],
            v_out: vec![TXOutput::new(SUBSIDY, to)],
        };
        tx.set_id();
        tx
    }

    pub fn new_utxo_tx(from: &str, to: &str, amount: i32, bc: &mut Blockchain) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let (acc, valid_outputs) = bc.find_spendable_outputs(from, amount);
        if acc < amount {
            panic!("ERROR: Not enough funds")
        }

        for (idx, outs) in valid_outputs.iter() {
            for out in outs {
                let input = TXInput::new(idx, *out, from);
                inputs.push(input);
            }
        }

        outputs.push(TXOutput::new(amount, to));
        if acc > amount {
            outputs.push(TXOutput::new(acc - amount, from))
        }

        let mut tx = Transaction{
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

    fn set_id(&mut self) {
        let mut hasher = Sha256::new();
        let data = bincode::serialize(&self)
            .expect("error serializing transaction");
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXInput {
    tx_id: String,
    v_out: i32,
    script_sig: String,
}

impl TXInput {
    pub fn new(tx_id: &str, v_out: i32, data: &str) -> TXInput {
        TXInput{
            tx_id: tx_id.to_string(),
            v_out,
            script_sig: data.to_string(),
        }
    }

    pub fn can_unlock_output(&self, unlocking_data: &str) -> bool {
        self.script_sig  == unlocking_data
    }

    pub fn tx_id(&self) -> &str {
        &self.tx_id[..]
    }

    pub fn v_out(&self) -> i32 {
        self.v_out
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXOutput {
    value: i32,
    script_pub_key: String,
}

impl TXOutput {
    pub fn new(amount: i32, to: &str) -> TXOutput {
        TXOutput{
            value: amount,
            script_pub_key: to.to_string(),
        }
    }

    pub fn can_be_unlocked(&self, unlocking_data: &str) -> bool {
        self.script_pub_key  == unlocking_data
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}
