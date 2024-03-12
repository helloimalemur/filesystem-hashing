use crate::snapshot::{FileMetadata, HashType};
use bytes::{BufMut, BytesMut};
use sha3::digest::block_buffer::Error;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::MutexGuard;
use std::{env, fs};

pub struct HashResult {
    pub check_sum: Vec<u8>,
    pub size: u64,
    pub ino: u64,
    pub ctime: i64,
    pub mtime: i64,
}
#[allow(unused)]
pub fn hash_files(
    path: &Path,
    file_hashes: &mut MutexGuard<HashMap<String, FileMetadata>>,
    hash_type: HashType,
) -> Result<(), Error> {
    let mut full_path = String::new();
    if path.starts_with("./") {
        if let Ok(cwd) = env::current_dir() {
            match cwd.to_str() {
                None => return Err(Error),
                Some(c) => full_path.push_str(c),
            }
            full_path.push('/');

            match path.to_str() {
                None => return Err(Error),
                Some(p) => match p.split("./").last() {
                    None => return Err(Error),
                    Some(p) => full_path.push_str(p),
                },
            }
        }
    } else {
        match path.to_str() {
            None => return Err(Error),
            Some(p) => {
                full_path.push_str(p);
            }
        }
    }

    let blacklist: Vec<&str> = vec!["/proc", "/tmp"];

    for entry in blacklist {
        if full_path.starts_with(entry) {
            return Err(Error);
        }
    }

    let mut size = 0u64;
    let mut ino = 0u64;
    let mut ctime = 0i64;
    let mut mtime = 0i64;

    if let Ok(metadata) = fs::metadata(full_path) {
        size = metadata.size();
        ctime = metadata.ctime();
        mtime = metadata.mtime();
        ino = metadata.ino();
    }

    let mut file_hash = BytesMut::new();

    if let Ok(file_handle) = fs::read(path) {
        let bytes = file_handle.as_slice();

        let byte_hash = match hash_type {
            HashType::Fast => hash_md5(Vec::from(bytes)),
            HashType::Full => hash_sha3(Vec::from(bytes)),
        };

        file_hash.put_slice(&byte_hash);
        drop(byte_hash);
    } else {
        return Err(Error);
    }

    match path.to_str() {
        None => return Err(Error),
        Some(p) => {
            file_hashes.insert(
                p.to_string(),
                FileMetadata {
                    path: p.to_string(),
                    check_sum: file_hash.to_vec(),
                    size,
                    ino,
                    ctime,
                    mtime,
                },
            );
        }
    }

    drop(file_hash);
    Ok(())
}

fn hash_sha3(bytes: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    let mut bytes_to_hash = BytesMut::new();

    bytes_to_hash.put_slice(&bytes);
    hasher.update(bytes_to_hash);
    hasher.finalize().to_vec()
}

fn hash_md5(bytes: Vec<u8>) -> Vec<u8> {
    md5::compute(bytes).to_vec()
}
