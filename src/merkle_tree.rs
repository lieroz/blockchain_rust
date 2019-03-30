use crypto::{digest::Digest, sha2::Sha256};

#[derive(Clone)]
struct MerkleNode {
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    data: Option<Vec<u8>>,
}

impl MerkleNode {
    pub fn new(
        left: Option<MerkleNode>,
        right: Option<MerkleNode>,
        data: Option<Vec<u8>>,
    ) -> MerkleNode {
        let mut new_node = MerkleNode {
            left: None,
            right: None,
            data: None,
        };
        let mut hasher = Sha256::new();
        let mut result: [u8; 32] = [0; 32];

        if left.is_none() && right.is_none() {
            hasher.input(&data.unwrap());
        } else {
            let mut prev_hashes = Vec::new();
            prev_hashes.extend(
                left.as_ref()
                    .unwrap()
                    .data
                    .as_ref()
                    .unwrap()
                    .iter()
                    .cloned(),
            );
            prev_hashes.extend(
                right
                    .as_ref()
                    .unwrap()
                    .data
                    .as_ref()
                    .unwrap()
                    .iter()
                    .cloned(),
            );
            hasher.input(&prev_hashes);
            new_node.left = Some(Box::new(left.unwrap()));
            new_node.right = Some(Box::new(right.unwrap()));
        }

        hasher.result(&mut result);
        new_node.data = Some(result.to_vec());
        new_node
    }
}

pub struct MerkleTree {
    root: Option<Box<MerkleNode>>,
}

impl MerkleTree {
    pub fn new(data: &mut Vec<Vec<u8>>) -> MerkleTree {
        let mut nodes = Vec::new();

        if data.len() % 2 != 0 {
            data.push(data[data.len() - 1].clone());
        }

        for datum in data.iter() {
            nodes.push(MerkleNode::new(None, None, Some(datum.clone())));
        }

        for _ in 0..(data.len() / 2) {
            let mut new_level = Vec::new();

            for i in (0..nodes.len()).step_by(2) {
                new_level.push(MerkleNode::new(
                    Some(nodes[i].clone()),
                    Some(nodes[i + 1].clone()),
                    None,
                ));
            }

            nodes = new_level;
        }

        MerkleTree {
            root: Some(Box::new(nodes[0].clone())),
        }
    }

    pub fn data(&self) -> Vec<u8> {
        self.root.as_ref().unwrap().data.as_ref().unwrap().clone()
    }
}
