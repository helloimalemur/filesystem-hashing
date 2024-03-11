use std::fs;
use std::fs::File;
use std::io::{Bytes, Read};
use std::path::Path;
use sha3::{Digest, Sha3_256};
use bytes::{BufMut, BytesMut};

pub fn hash_file(path: &Path) -> BytesMut {
    let mut hasher = Sha3_256::new();
    let mut bytes_to_hash = BytesMut::new();
    let mut file_hash = BytesMut::new();
    if let Ok(file_handle) = fs::read(path) {
        let bytes = file_handle.as_slice();
        bytes_to_hash.put_slice(bytes);
        hasher.update(bytes_to_hash);
        file_hash.put_slice(hasher.finalize().as_ref());
    }
    file_hash
}
