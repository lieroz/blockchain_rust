use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use crate::utxo_set::UTXOSet;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::cell::RefCell;

const NODE_VERSION: i32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    version: i32,
    best_height: i32,
    addr_from: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockWrapper {
    addr_from: String,
    block: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Inv {
    addr_from: String,
    kind: String,
    items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetData {
    addr_from: String,
    kind: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxWrapper {
    pub addr_from: String,
    pub tx: Vec<u8>,
}

pub struct Server {
    node_id: String,
    node_address: String,
    mining_address: String,
    known_nodes: Mutex<RefCell<Vec<String>>>,
    blocks_in_transit: Mutex<RefCell<Vec<String>>>,
    mempool: Mutex<RefCell<HashMap<String, Transaction>>>,
    bc: Mutex<RefCell<Blockchain>>
}

impl Server {
    fn request_blocks(&self) {
        for node in self.known_nodes.lock().unwrap().borrow().iter() {
            self.send_get_blocks(node);
        }
    }

    fn send_data(&self, address: &str, request: &[u8]) {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                stream.write(request).unwrap();
                stream.flush().unwrap();
            },
            Err(e) => {
                println!("{} is not available: {}", address, e);
                let mut updated_nodes = Vec::new();

                for node in self.known_nodes.lock().unwrap().borrow().iter() {
                    if node != address {
                        updated_nodes.push(node.clone());
                    }
                }

                *self.known_nodes.lock().unwrap().borrow_mut() = updated_nodes;
            },
        };
    }

    fn send_addr(&self, address: &str) {
        let cmd = b"addr\n";
        let mut nodes = self.known_nodes.lock().unwrap().borrow().clone();
        nodes.push(self.node_address.clone());
        let payload = bincode::serialize(&nodes).unwrap();
        let mut request: Vec<u8> = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn send_block(&self, address: &str, b: &Block) {
        let cmd = b"block\n";
        let data = BlockWrapper{
            addr_from: self.node_address.clone(),
            block: b.serialize(),
        };
        let payload = bincode::serialize(&data).unwrap();
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn send_inv(&self, address: &str, kind: &str, items: Vec<String>) {
        let cmd = b"inv\n";
        let data = Inv{
            addr_from: self.node_address.clone(),
            kind: kind.to_string(),
            items,
        };
        let payload = bincode::serialize(&data).unwrap();
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn send_get_blocks(&self, address: &str) {
        let cmd = b"getblocks\n";
        let mut request: Vec<u8> = Vec::new();
        request.extend(cmd);
        request.extend(self.node_address.as_bytes());
        self.send_data(address, &request);
    }

    fn send_get_data(&self, address: &str, kind: &str, id: &str) {
        let cmd = b"getdata\n";
        let data = GetData{
            addr_from: self.node_address.clone(),
            kind: kind.to_string(),
            id: id.to_string(),
        };
        let payload = bincode::serialize(&data).unwrap();
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    pub fn send_tx(&self, address: &str, tx: &Transaction) {
        let cmd = b"tx\n";
        let data = TxWrapper{
            addr_from: self.node_address.clone(),
            tx: tx.serialize(),
        };
        let payload = bincode::serialize(&data).unwrap();
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn send_version(&self, address: &str, height: i32) {
        let cmd = b"version\n";
        let version = Version{
            version: NODE_VERSION,
            best_height: height,
            addr_from: self.node_address.clone()
        };
        let payload = bincode::serialize(&version).unwrap();
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn handle_addr(&self, request: &[u8]) {
        let payload: Vec<String> = bincode::deserialize(request).unwrap();
        self.known_nodes.lock().unwrap().borrow_mut().extend(payload);
        println!("There are {} known nodes now!", self.known_nodes.lock().unwrap().borrow().len());
        self.request_blocks();
    }

    fn handle_block(&self, request: &[u8]) {
        let payload: BlockWrapper = bincode::deserialize(request).unwrap();
        let block = Block::deserialize(payload.block);
        println!("Received a new block!");

        self.bc.lock().unwrap().borrow_mut().add_block(&block);
        println!("Added block: {}", block.hash());

        if self.blocks_in_transit.lock().unwrap().borrow().len() > 0 {
            let block_hash = self.blocks_in_transit.lock().unwrap().borrow()[0].clone();
            self.send_get_data(&payload.addr_from, "block", &block_hash);
            self.blocks_in_transit.lock().unwrap().borrow_mut().remove(0);
        } else {
            let mut utxo_set = UTXOSet::new(&self.node_id);
            utxo_set.reindex(&self.node_id, &mut self.bc.lock().unwrap().borrow_mut());
        }
    }

    fn handle_inv(&self, request: &[u8]) {
        let payload: Inv = bincode::deserialize(request).unwrap();
        println!("Received inventory with {} {}", payload.items.len(), payload.kind);

        match payload.kind.as_ref() {
            "block" =>  {
                *self.blocks_in_transit.lock().unwrap().borrow_mut() = payload.items.iter().rev().cloned().collect();
                let block_hash = self.blocks_in_transit.lock().unwrap().borrow()[0].clone();
                self.send_get_data(&payload.addr_from, "block", &block_hash);
                let mut new_in_transit = Vec::new();

                for b in self.blocks_in_transit.lock().unwrap().borrow().iter() {
                    if block_hash != *b {
                        new_in_transit.push(b.clone());
                    }
                }

                *self.blocks_in_transit.lock().unwrap().borrow_mut() = new_in_transit;
            },
            "tx" => {
                let tx_id = payload.items[0].clone();

                if !self.mempool.lock().unwrap().borrow().contains_key(&tx_id) {
                    self.send_get_data(&payload.addr_from, "tx", &tx_id);
                }
            },
            kind => panic!("Unknown payload kind: {}", kind),
        }
    }

    fn handle_get_blocks(&self, request: &[u8]) {
        let addr_from = String::from_utf8_lossy(request);
        let blocks = self.bc.lock().unwrap().borrow_mut().get_block_hashes();
        self.send_inv(&addr_from, "block", blocks);
    }

    fn handle_get_data(&self, request: &[u8]) {
        let payload: GetData = bincode::deserialize(request).unwrap();

        match payload.kind.as_ref() {
            "block" => {
                let block = self.bc.lock().unwrap().borrow_mut().get_block(&payload.id);
                self.send_block(&payload.addr_from, &block);
            },
            "tx" => {
                let tx = self.mempool.lock().unwrap().borrow().get(&payload.id).unwrap().clone();
                self.send_tx(&payload.addr_from, &tx);
                // self.mempool.lock().unwrap().borrow_mut().remove(tx.id());
            },
            kind => panic!("Unknown payload kind: {}", kind),
        }
    }

    fn handle_tx(&self, request: &[u8]) {
        let payload: TxWrapper = bincode::deserialize(request).unwrap();
        let tx = Transaction::deserialize(payload.tx);
        self.mempool.lock().unwrap().borrow_mut().insert(tx.id().to_string(), tx.clone());

        if self.node_address == self.known_nodes.lock().unwrap().borrow()[0] {
            for node in self.known_nodes.lock().unwrap().borrow().iter() {
                if *node != self.node_address && *node != payload.addr_from {
                    self.send_inv(node, "tx", vec![tx.id().to_string()]);
                }
            }
        } else {
            if self.mempool.lock().unwrap().borrow().len() >= 2 && !self.mining_address.is_empty() {
                loop {
                    let mut txs = Vec::new();

                    for (_, v) in self.mempool.lock().unwrap().borrow().iter() {
                        if self.bc.lock().unwrap().borrow_mut().verify_transaction(v) {
                            txs.push(v.clone());
                        }
                    }

                    if txs.is_empty() {
                        println!("All transactions are invalid! Waiting for new ones...");
                        return
                    }

                    txs.push(Transaction::new_coin_base_tx(&self.mining_address, ""));
                    let new_block = self.bc.lock().unwrap().borrow_mut().mine_block(txs.clone());
                    let mut utxo_set = UTXOSet::new(&self.node_id);
                    utxo_set.reindex(&self.node_id, &mut self.bc.lock().unwrap().borrow_mut());

                    for tx in txs {
                        self.mempool.lock().unwrap().borrow_mut().remove(tx.id());
                    }

                    for node in self.known_nodes.lock().unwrap().borrow().iter() {
                        if *node != self.node_address {
                            self.send_inv(node, "block", vec![new_block.hash().to_string()]);
                        }
                    }

                    if self.mempool.lock().unwrap().borrow().is_empty() {
                        break
                    }
                }
            }
        }
    }

    fn handle_version(&self, request: &[u8]) {
        let payload: Version = bincode::deserialize(request).unwrap();
        let my_best_height = self.bc.lock().unwrap().borrow_mut().get_best_height();
        let foreigner_best_height = payload.best_height;

        if my_best_height < foreigner_best_height {
            self.send_get_blocks(&payload.addr_from);
        } else {
            self.send_version(&payload.addr_from, my_best_height);
        }

        // self.send_addr(&payload.addr_from);

        if !self.is_known_node(&payload.addr_from) {
            self.known_nodes.lock().unwrap().borrow_mut().push(payload.addr_from);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut cmd = String::new();
        let mut request: Vec<u8> = Vec::new();
        let mut flag = false;

        loop {
            let mut buf = [0; 512];
            match stream.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }

                    if !flag {
                        for (i, b) in buf.iter().enumerate() {
                            if *b == b'\n' {
                                flag = true;
                                cmd.push_str(std::str::from_utf8(&buf[..i]).unwrap());

                                if i + 1 != n {
                                    request.extend(buf[i + 1..n].iter());
                                }

                                break;
                            }
                        }
                    } else {
                        request.extend(buf.iter());
                    }
                }
                Err(e) => panic!("Error while reading from socket: {}", e),
            }
        }

        match cmd.as_ref() {
            "addr" => self.handle_addr(&request),
            "block" => self.handle_block(&request),
            "inv" => self.handle_inv(&request),
            "getblocks" => self.handle_get_blocks(&request),
            "getdata" => self.handle_get_data(&request),
            "tx" => self.handle_tx(&request),
            "version" => self.handle_version(&request),
            cmd => panic!("Unknown command: {}", cmd),
        }
    }

    pub fn start(node_id: &str, miner_address: &str) {
        let node_address = format!("127.0.0.1:{}", node_id);
        let server = Arc::new(Server {
            node_id: node_id.to_string(),
            node_address,
            mining_address: miner_address.to_string(),
            known_nodes: Mutex::new(RefCell::new(vec!["127.0.0.1:3000".to_string()])),
            blocks_in_transit: Mutex::new(RefCell::new(Vec::new())),
            mempool: Mutex::new(RefCell::new(HashMap::new())),
            bc: Mutex::new(RefCell::new(Blockchain::new(node_id))),
        });

        if server.node_address != server.known_nodes.lock().unwrap().borrow()[0] {
            let height = server.bc.lock().unwrap().borrow_mut().get_best_height();
            server.send_version(&server.known_nodes.lock().unwrap().borrow()[0], height);
        }

        let listener = TcpListener::bind(format!("127.0.0.1:{}", node_id)).unwrap();
        for stream in listener.incoming() {
            let server = Arc::clone(&server);
            thread::spawn(move || {
                let stream = stream.unwrap();
                server.handle_connection(stream);
            });
        }
    }

    fn is_known_node(&self, address: &str) -> bool {
        for node in self.known_nodes.lock().unwrap().borrow().iter() {
            if node == address {
                return true
            }
        }
        false
    }
}
