use x25519_dalek::{StaticSecret, PublicKey};
use hkdf::Hkdf;
use sha2::Sha256;
use chacha20poly1305::{
    XChaCha20Poly1305,
    KeyInit
};

pub struct Keypair {
    pub private_key: StaticSecret,
    pub public_key: PublicKey
}

pub fn generate_assymetric_keypair() -> Keypair {
    let private_key = StaticSecret::random_from_rng(rand::thread_rng());
    let public_key = PublicKey::from(&private_key);

    Keypair {
        private_key,
        public_key
    }
}

pub fn generate_cipher(public_key: PublicKey, private_key: StaticSecret) -> XChaCha20Poly1305 {
    let shared_key = private_key.diffie_hellman(&public_key);
    let hk = Hkdf::<Sha256>::new(Some(b"chat-app-salt"), shared_key.as_bytes());
    let mut hash_buffer = [0u8; 32];
    hk.expand(b"encryption_key", &mut hash_buffer).unwrap();

    XChaCha20Poly1305::new_from_slice(&hash_buffer).unwrap()
}
