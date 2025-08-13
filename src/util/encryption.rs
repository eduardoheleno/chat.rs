use rsa::{RsaPrivateKey, RsaPublicKey};

pub struct Keypair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey
}

pub fn generate_assymetric_keypair() -> Keypair {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);

    Keypair {
        private_key,
        public_key
    }
}
