use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use rand::{Rng, thread_rng};
use crate::hasher::hash_files;

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
    fn new(path: &Path) -> Snapshot {
        let mut rand = thread_rng();
        let uuid_int: i128 = rand.gen();
        let uuid = uuid_int.to_string();

        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();

        let mut file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>> = Arc::new(Mutex::new(HashMap::new()));

        for path in file_paths {
            if let Ok(p) = path {
                if p.path().is_file() {
                    if let Ok(result) = hash_files(p.path(), file_hashes.clone()) {
                    }
                }
            }
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

        // let test_snap = Snapshot::new(Path::new("/etc"));
        let test_snap = Snapshot::new(Path::new("/home/foxx/Downloads/"));
        // println!("{}", test_snap.file_hashes.len());
        //

        println!("Sample: {:#?}", test_snap.file_hashes.lock().unwrap().iter().last());

        println!("Files: {}", test_snap.file_hashes.lock().unwrap().len());

        // for fi in test_snap.file_hashes.iter() {
        //     println!("{}", fi.0)
        // }

        // assert_eq!(result, 4);
    }
}
