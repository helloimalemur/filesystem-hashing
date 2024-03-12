use crate::hasher::hash_files;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Clone, Copy)]
pub enum HashType {
    Fast,
    Full,
}
#[derive(Debug)]
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    uuid: String,
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
    pub fn new(path: &Path, hash_type: HashType) -> Snapshot {
        let mut rand = thread_rng();
        let uuid_int: i128 = rand.gen();
        let uuid = uuid_int.to_string();
        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();
        let file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut hashers: Vec<JoinHandle<()>> = vec![];

        for p in file_paths.into_iter().flatten() {
            if p.path().is_file() {
                let bind = file_hashes.clone();

                let handle = thread::spawn(move || {
                    let mut binding = bind.lock();
                    let ht = binding.as_mut().unwrap();
                    let _ = hash_files(p.path(), ht, hash_type);
                });
                hashers.push(handle)
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
    use super::*;
    use std::path::Path;
    use std::time::SystemTime;

    #[test]
    fn create_snapshot() {
        // let test_snap = Snapshot::new(Path::new("/home/foxx/hashtest/"), HashType::Full);

        // let test_snap = Snapshot::new(Path::new("/"), HashType::Fast);
        // let test_snap = Snapshot::new(Path::new("/var/"), HashType::Fast); // danger
        // let test_snap = Snapshot::new(Path::new("/etc/"), HashType::Fast); // safe
        // let test_snap = Snapshot::new(Path::new("/etc/"), HashType::Full); // safe
        let start = SystemTime::now();
        let test_snap = Snapshot::new(Path::new("/home/foxx/Downloads/"), HashType::Fast);
        // let test_snap = Snapshot::new(Path::new("/home/foxx/hashtest/"), HashType::Fast);
        let stop = SystemTime::now();
        let lapsed = stop.duration_since(start).unwrap();
        println!("{:?}", lapsed);

        let start2 = SystemTime::now();
        let test_snap = Snapshot::new(Path::new("/home/foxx/Downloads/"), HashType::Full);
        // let test_snap = Snapshot::new(Path::new("/home/foxx/hashtest/"), HashType::Full);
        let stop2 = SystemTime::now();
        let lapsed2 = stop2.duration_since(start2).unwrap();
        println!("{:?}", lapsed2);


       // let test_snap = Snapshot::new(Path::new("/home/foxx/hashtest/"), HashType::Fast);
       // let test_snap = Snapshot::new(Path::new("/home/foxx/Documents/pcidocs/"), HashType::Fast);
       // let test_snap = Snapshot::new(Path::new("/home/foxx/Documents/pci_lynis/"), HashType::Fast);

        // println!(
        //     "Sample: {:#?}",
        //     test_snap.file_hashes.lock().unwrap().iter().last()
        // );
        println!("Files: {}", test_snap.file_hashes.lock().unwrap().len());

        // for fi in test_snap.file_hashes.iter() {
        //     println!("{}", fi.0)
        // }
    }
}
