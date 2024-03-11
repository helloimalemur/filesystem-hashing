use std::{env, fs};
use std::io::{Read};
use std::path::Path;
use sha3::{Digest, Sha3_256};
use bytes::{BufMut, BytesMut};

pub fn hash_file(path: &Path) -> (String, usize, Vec<u8>) {
    let mut hasher = Sha3_256::new();
    let mut bytes_to_hash = BytesMut::new();
    let mut file_hash = BytesMut::new();

    let mut full_path = String::new();
    if path.starts_with("./") {
        if let Ok(cwd) = env::current_dir() {
            full_path.push_str(cwd.to_str().unwrap());
            full_path.push_str("/");
            full_path.push_str(path.to_str().unwrap().split("./").last().unwrap());
        }
    } else {
        full_path.push_str(path.to_str().unwrap());
    }

    

    if let Ok(file_handle) = fs::read(path) {
        let bytes = file_handle.as_slice();
        bytes_to_hash.put_slice(bytes);
        hasher.update(bytes_to_hash);
        file_hash.put_slice(hasher.finalize().as_ref());
    }

    (path.to_str().unwrap().to_string(), file_hash.len(), file_hash.to_vec())
}
