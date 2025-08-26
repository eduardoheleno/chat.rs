use std::fs::{self, File};
use std::io::prelude::*;
use std::env;
use std::path::PathBuf;

const SERVICE_NAME: &str = "nossochat-service";

pub fn save_private_key(user_email: String, private_key_bytes: &[u8; 32]) {
    let home_dir = env::var("HOME")
        .map(PathBuf::from)
        .expect("Couldn't determine home directory");
    let folder_path = home_dir.join(format!(".{}/", SERVICE_NAME));
    fs::create_dir_all(&folder_path).expect("Couldn't create app's folder");

    let file_path = folder_path.join(format!("{}_private.bin", user_email));
    let mut file = File::create(file_path).expect("Couldn't create secret file");
    file.write_all(private_key_bytes).expect("Couldn't write private key into file");
}

pub fn get_private_key(user_email: String) -> Vec<u8> {
    let home_dir = env::var("HOME")
        .map(PathBuf::from)
        .expect("Couldn't determine home directory");
    let folder_path = home_dir.join(format!(".{}/", SERVICE_NAME));
    fs::create_dir_all(&folder_path).expect("Couldn't create app's folder");

    let file_path = folder_path.join(format!("{}_private.bin", user_email));
    fs::read(file_path).expect("Couldn't read private file")
}
