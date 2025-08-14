use keyring::Entry;

const SERVICE_NAME: &str = "nossochat-service";

pub fn save_private_key(user_key: String, private_key_bytes: Vec<u8>) {
    let entry = Entry::new(SERVICE_NAME, &user_key).expect("Couldn't create keyring entry");
    entry.set_secret(&private_key_bytes).expect("Couldn't save private key on keyring");
}

pub fn get_private_key(user_key: String) {
    let entry = Entry::new(SERVICE_NAME, &user_key).expect("Couldn't create keyring entry");
    let test = entry.get_secret().unwrap();
    println!("{:?}", test);
}
