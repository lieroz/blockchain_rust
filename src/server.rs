use crate::block::Block;
use crate::transaction::Transaction;
use crate::blockchain::Blockchain;

use std::net::TcpStream;
use std::net::TcpListener;
use std::collections::HashMap;
use std::io::{Read, Write};

const NODE_VERSION: i32 = 1;
const COMMAND_LENGTH: i32 = 12;

pub struct Server {
    node_address: String,
    mining_address: String,
    known_nodes: Vec<String>,
    blocks_in_transit: Vec<Vec<u8>>,
    mempool: HashMap<String, Transaction>,
}

impl Server {
    fn send_data(&self, address: &str, request: &[u8]) {
    }

    fn send_addr(&self, address: &str) {
        let mut nodes = self.known_nodes.clone();
        nodes.push(self.node_address.clone());
        let payload = bincode::serialize(&nodes).unwrap();
        let cmd = b"addr\n";
        let mut request = Vec::new();
        request.extend(cmd);
        request.extend(payload);
        self.send_data(address, &request);
    }

    fn send_block() {
    }

    fn send_inv() {
    }

    fn send_get_data() {
    }

    fn send_tx() {
    }

    fn send_version() {
    }

    fn handle_addr() {
    }

    fn handle_block() {
    }

    fn handle_inv() {
    }

    fn handle_get_blocks() {
    }

    fn handle_get_data() {
    }

    fn handle_tx() {
    }

    fn handle_version() {
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut msg = String::new();
        loop {
            let mut buf = [0; 512];
            match stream.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    msg.push_str(std::str::from_utf8(&buf[..n]).unwrap());
                },
                Err(e) => panic!("Error while reading from socket: {}", e),
            }
        }

        println!("{:?}", msg.find("\r\n"));
    }

    pub fn start(node_id: &str, miner_address: &str) {
        let node_address = format!("127.0.0.1:{}", node_id);
        let server = Server{
            node_address,
            mining_address: miner_address.to_string(),
            known_nodes: vec!["127.0.0.1:3000".to_string()],
            blocks_in_transit: Vec::new(),
            mempool: HashMap::new(),
        };
        let listener = TcpListener::bind(format!("127.0.0.1:{}", node_id)).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            server.handle_connection(stream);
        }
    }
}
