use crate::snapshot::FileMetadata;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::MutexGuard;
use std::{env, fs};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum HashType {
    MD5,
    SHA3,
    BLAKE3,
}

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
    verbose: bool,
) -> Result<(), Error> {
    let mut full_path = String::new();
    if path.starts_with("./") {
        if let Ok(cwd) = env::current_dir() {
            match cwd.to_str() {
                None => return Err(anyhow!("cannot parse path")),
                Some(c) => full_path.push_str(c),
            }
            full_path.push('/');

            match path.to_str() {
                None => return Err(anyhow!("cannot parse path")),
                Some(p) => match p.split("./").last() {
                    None => return Err(anyhow!("cannot parse path")),
                    Some(p) => full_path.push_str(p),
                },
            }
        }
    } else {
        match path.to_str() {
            None => return Err(anyhow!("cannot parse path")),
            Some(p) => {
                full_path.push_str(p);
            }
        }
    }

    let black_list: Vec<&str> = vec![];

    for entry in black_list {
        if full_path.starts_with(entry) {
            return Err(anyhow!("cannot parse path"));
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

    let mut file_hash: Vec<u8> = Vec::new();
    let mut file_buffer: Vec<u8> = Vec::new();

    if verbose {
        println!("{}", path.to_str().unwrap())
    }
    
    let byte_hash: Result<Vec<u8>, Error> = match hash_type {
        HashType::MD5 => hash_md5(path),
        HashType::SHA3 => hash_sha3(path),
        HashType::BLAKE3 => hash_blake3(path),
    };

    match path.to_str() {
        None => return Err(anyhow!("cannot parse path")),
        Some(p) => {
            file_hashes.insert(
                p.to_string(),
                FileMetadata {
                    path: p.to_string(),
                    check_sum: byte_hash?,
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

fn hash_sha3(bytes: &Path) -> Result<Vec<u8>, Error> {
    let mut hasher = Sha3_256::new();
    if let Ok(mut f) = File::open(bytes) {
        let chunk_size = 0x4000;
        if let Ok(meta) = f.metadata() {
            if meta.is_file() {
                loop {
                    let mut chunk = Vec::with_capacity(chunk_size);
                    let n = std::io::Read::by_ref(&mut f)
                        .take(chunk_size as u64)
                        .read_to_end(&mut chunk)?;
                    if n == 0 {
                        break;
                    }
                    sha3::digest::Update::update(&mut hasher, chunk.by_ref());
                    if n < chunk_size {
                        break;
                    }
                }
            }
        }
    }
    Ok(hasher.finalize().to_vec())
}

fn hash_md5(bytes: &Path) -> Result<Vec<u8>, Error> {
    let mut hasher = md5::Context::new();
    if let Ok(mut f) = File::open(bytes) {
        let chunk_size = 0x4000;
        if let Ok(meta) = f.metadata() {
            if meta.is_file() {
                loop {
                    let mut chunk = Vec::with_capacity(chunk_size);
                    let n = std::io::Read::by_ref(&mut f)
                        .take(chunk_size as u64)
                        .read_to_end(&mut chunk)?;
                    if n == 0 {
                        break;
                    }
                    hasher.consume(chunk);
                    if n < chunk_size {
                        break;
                    }
                }
            }
        }
    }
    Ok(hasher.compute().0.to_vec())
}

fn hash_blake3(bytes: &Path) -> Result<Vec<u8>, Error> {
    let mut hasher = blake3::Hasher::new();
    if let Ok(mut f) = File::open(bytes) {
        let chunk_size = 0x4000;
        if let Ok(meta) = f.metadata() {
            if meta.is_file() {
                loop {
                    let mut chunk = Vec::with_capacity(chunk_size);
                    let n = std::io::Read::by_ref(&mut f)
                        .take(chunk_size as u64)
                        .read_to_end(&mut chunk)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(chunk.as_ref());
                    if n < chunk_size {
                        break;
                    }
                }
            }
        }
    }
    Ok(hasher.finalize().as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use sha3::Digest;

    #[test]
    fn blake3() {
        let test_string = "aprettylongteststring".as_bytes();
        let hashed = blake3::hash(test_string).as_bytes().to_vec();
        assert_eq!(
            hashed,
            [
                0xFD, 0x5F, 0x22, 0xE8, 0x95, 0x82, 0x18, 0xD6, 0x9A, 0x96, 0xAC, 0x77, 0xCD, 0xCD,
                0xAA, 0xA7, 0x51, 0xCE, 0x81, 0xF3, 0x04, 0x86, 0xC8, 0x49, 0xA6, 0xD7, 0x66, 0x81,
                0x68, 0xDB, 0x22, 0x2D,
            ]
        )
    }

    #[test]
    fn md5() {
        let test_string = "adifferentbutstillprettylongteststring".as_bytes();
        let hashed = md5::compute(test_string).to_vec();
        // println!("{:#04X?}", hashed);
        assert_eq!(
            hashed,
            [
                0x6C, 0x39, 0x5D, 0xC4, 0xC5, 0x81, 0xAE, 0x7A, 0x55, 0x74, 0xC4, 0x5B, 0xE3, 0xFB,
                0x92, 0x58
            ]
        )
    }

    #[test]
    fn sha3() {
        let test_string =
            "adifferentbutstillprettylongteststringwithaslightlydifferentcontent".as_bytes();
        let hashed = sha3::Sha3_256::digest(test_string).to_vec();
        println!("{:#04X?}", hashed);
        assert_eq!(
            hashed,
            [
                0xA1, 0x55, 0xE2, 0x73, 0x63, 0x51, 0x36, 0xC5, 0x25, 0xFB, 0x36, 0xA8, 0x81, 0xD6,
                0x02, 0x21, 0xCC, 0xC5, 0x48, 0x9B, 0xE7, 0x18, 0xCC, 0x57, 0xCE, 0x66, 0xBA, 0x78,
                0xBA, 0x26, 0x33, 0x7E,
            ]
        )
    }
}
