use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use rand::{Rng, thread_rng};
use crate::hasher::hash_files;

#[derive(Clone, Copy)]
pub enum HashType {
    Fast,
    Full
}
#[derive(Debug)]
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    uuid: String
}
#[derive(Debug)]
pub struct FileMetadata {
    pub path: String,
    pub check_sum: Vec<u8>,
    pub size: u64,
    pub ino: u64,
    pub ctime: i64,
    pub mtime: i64,
}

impl Snapshot {
    fn new(path: &Path, hash_type: HashType) -> Snapshot {
        let mut rand = thread_rng();
        let uuid_int: i128 = rand.gen();
        let uuid = uuid_int.to_string();
        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();
        let mut file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut hashers: Vec<JoinHandle<()>> = vec![];

        for path in file_paths {
            if let Ok(p) = path {
                if p.path().is_file() {
                    let bind = file_hashes.clone();

                    let handle = thread::spawn(move || {
                        let _ = hash_files(p.path(), bind, hash_type);
                    });
                    hashers.push(handle)
                }
            }
        }

        for handle in hashers {
            handle.join().expect("could not join handle")
        }

        Snapshot { file_hashes, uuid }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn create_snapshot() {
        // let test_snap = Snapshot::new(Path::new("/"));
        // let test_snap = Snapshot::new(Path::new("/etc"));
        // let test_snap = Snapshot::new(Path::new("/home/foxx/IdeaProjects"), HashType::Full);

        // let test_snap = Snapshot::new(Path::new("/home/foxx/hashtest/"), HashType::Full);
        let test_snap = Snapshot::new(Path::new("/home/foxx/hashtest/"), HashType::Fast);

        println!("Sample: {:#?}", test_snap.file_hashes.lock().unwrap().iter().last());
        println!("Files: {}", test_snap.file_hashes.lock().unwrap().len());

        // for fi in test_snap.file_hashes.iter() {
        //     println!("{}", fi.0)
        // }

        // assert_eq!(result, 4);
    }
}
