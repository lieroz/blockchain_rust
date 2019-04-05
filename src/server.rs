use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;

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

pub struct Server {
    node_id: String,
    node_address: String,
    mining_address: String,
    known_nodes: Mutex<RefCell<Vec<String>>>,
    blocks_in_transit: Vec<Vec<u8>>,
    mempool: HashMap<String, Transaction>,
}

impl Server {
    fn send_data(&self, address: &str, request: &[u8]) {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                stream.write(request).unwrap();
                stream.flush().unwrap();
            },
            Err(e) => {
                println!("{} is not available", address);
                let mut updated_nodes = Vec::new();

                for node in self.known_nodes.lock().unwrap().borrow().iter() {
                    if node != address {
                        updated_nodes.push(node.clone());
                    }
                }

                *self.known_nodes.lock().unwrap() = RefCell::new(updated_nodes);
            },
        };
    }

    fn send_addr(&self, address: &str) {}

    fn send_block() {}

    fn send_inv() {}

    fn send_get_blocks(&self, address: &str) {}

    fn send_get_data() {}

    fn send_tx() {}

    fn send_version(&self, address: &str, height: i32) {
        let cmd = b"version\n";
        let version = Version{
            version: NODE_VERSION,
            best_height: height,
            addr_from: address.to_string()
        };
        let payload = bincode::serialize(&version).unwrap();
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn handle_addr() {}

    fn handle_block() {}

    fn handle_inv() {}

    fn handle_get_blocks() {}

    fn handle_get_data() {}

    fn handle_tx() {}

    fn handle_version(&self, request: &[u8]) {
        let payload: Version = bincode::deserialize(request).unwrap();
        let mut bc = Blockchain::new(&self.node_id);
        let my_best_height = bc.get_best_height();
        let foreigner_best_height = payload.best_height;

        if my_best_height < foreigner_best_height {
            self.send_get_blocks(&payload.addr_from);
        } else {
            self.send_version(&payload.addr_from, my_best_height);
        }

        if !self.is_known_node(&payload.addr_from) {
            self.known_nodes.lock().unwrap().borrow_mut().push(payload.addr_from);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut cmd = String::new();
        let mut request: Vec<u8> = Vec::new();

        'reader: loop {
            let mut buf = [0; 512];
            match stream.read(&mut buf) {
                Ok(n) => {
                    for (i, b) in buf.iter().enumerate() {
                        if *b == b'\n' {
                            cmd.push_str(std::str::from_utf8(&buf[..i]).unwrap());
                            request.extend(buf[i + 1..n].iter());
                            continue 'reader;
                        }
                    }

                    if n == 0 {
                        break;
                    }
                }
                Err(e) => panic!("Error while reading from socket: {}", e),
            }
        }

        match cmd.as_ref() {
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
            blocks_in_transit: Vec::new(),
            mempool: HashMap::new(),
        });

        if server.node_address != server.known_nodes.lock().unwrap().borrow()[0] {
            let height = Blockchain::new(node_id).get_best_height();
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
