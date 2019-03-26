use crypto::{digest::Digest, ripemd160::Ripemd160, sha2::Sha256};
use ring::{
    rand,
    signature::{self, KeyPair},
};

const VERSION: u8 = 0;
pub const ADDRESS_CHECKSUM_LEN: usize = 4;

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pkcs8_bytes: Vec<u8>,
}

impl Wallet {
    pub fn new() -> Wallet {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes =
            signature::Ed25519KeyPair::generate_pkcs8(&rng).expect("error generating pkcs8 bytes");
        Wallet {
            pkcs8_bytes: pkcs8_bytes.as_ref().to_vec(),
        }
    }

    pub fn get_address(&self) -> String {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(
            self.pkcs8_bytes.as_ref(),
        ))
        .expect("error getting key pair from bytes");
        let pub_key_hash = Self::hash_pub_key(key_pair.public_key().as_ref());
        let mut payload = vec![VERSION];
        payload.extend(pub_key_hash);
        let checksum = Self::checksum(&payload);
        payload.extend(checksum);
        bs58::encode(&payload).into_string()
    }

    pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
        let mut sha2_hasher = Sha256::new();
        sha2_hasher.input(pub_key);
        let mut ripemd_hasher = Ripemd160::new();
        let mut hash: [u8; 32] = [0; 32];
        sha2_hasher.result(&mut hash);
        let mut result: [u8; 20] = [0; 20];
        ripemd_hasher.input(&hash);
        ripemd_hasher.result(&mut result);
        result.to_vec()
    }

    pub fn validate_address(address: &str) -> bool {
        let pub_key_hash = bs58::decode(address)
            .into_vec()
            .expect("error decoding address using base 58");
        let size = pub_key_hash.len() - ADDRESS_CHECKSUM_LEN;
        let actual_checksum = pub_key_hash[size..].to_vec();
        let version = pub_key_hash[0];
        let pub_key_hash = pub_key_hash[1..size].to_vec();
        let mut payload = vec![version];
        payload.extend(pub_key_hash);
        let target_checksum = Self::checksum(&payload);
        actual_checksum
            .iter()
            .zip(target_checksum.iter())
            .filter(|&(a, b)| a == b)
            .count()
            == 4
    }

    fn checksum(payload: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        let mut checksum: [u8; 32] = [0; 32];
        hasher.input(payload);
        hasher.result(&mut checksum);
        hasher.reset();
        hasher.input(&checksum);
        hasher.result(&mut checksum);
        checksum[..ADDRESS_CHECKSUM_LEN].to_vec()
    }

    pub fn public_key(&self) -> Vec<u8> {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(
            self.pkcs8_bytes.as_ref(),
        ))
        .expect("error getting key pair from bytes");
        key_pair.public_key().as_ref().to_vec()
    }
}
