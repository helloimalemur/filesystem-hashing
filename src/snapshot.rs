use crate::hasher::{hash_files, HashType};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;
use chrono::Utc;


#[derive(Debug)]
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64
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

impl Default for FileMetadata {
    fn default() -> Self {
        FileMetadata {
            path: "".to_string(),
            check_sum: vec![],
            size: 0,
            ino: 0,
            ctime: 0,
            mtime: 0,
        }
    }
}

impl Snapshot {
    pub fn new(path: &Path, hash_type: HashType) -> Snapshot {
        let root_path = match path.to_str() {
            None => {"".to_string()}
            Some(p) => {p.to_string()}
        };
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

        Snapshot { file_hashes, root_path, hash_type, uuid, date_created: Utc::now().timestamp() }
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Snapshot { file_hashes: Arc::new(Mutex::new(HashMap::new())), root_path: "".to_string(), hash_type: HashType::BLAKE3, uuid: "".to_string(), date_created: 0 }
    }
}

enum SnapshotChangeType {
    None,
    Created,
    Deleted,
    Changed
}

#[derive(Debug)]
pub struct SnapshotCompareResult {
    created: Vec<String>,
    deleted: Vec<String>,
    changed: Vec<String>
}

pub fn compare(left: Snapshot, right: Snapshot) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    let mut success = true;
    let mut created: Vec<String> = vec![];
    let mut deleted: Vec<String> = vec![];
    let mut changed: Vec<String> = vec![];



    match left.file_hashes.lock() {
        Ok(mut left_lock) => {

            // for each entry in the hash list
            for left_entry in left_lock.iter() {


                match right.file_hashes.lock() {
                    Ok(curr_lock) => {
                        // check for deletion
                        if !curr_lock.contains_key(left_entry.0) {
                            deleted.push(left_entry.0.to_string());
                        }

                        match curr_lock.get(left_entry.0) {
                            Some(right_entry) => {

                                // check for mis-matching checksum
                                if !right_entry.check_sum.eq(&left_entry.1.check_sum) {
                                    changed.push(right_entry.path.to_string());
                                }

                            }
                            None => {success = false}
                        }

                    }
                    Err(_) => {success = false}

                }

            }

        }
        Err(_) => {success = false}
    }

    match right.file_hashes.lock() {
        Ok(e) => {
            for right_entry in e.iter() {
                // check for file creations
                if left.file_hashes.lock().unwrap().get(right_entry.0).is_none() {
                    created.push(right_entry.0.to_string());
                }
            }
        }
        Err(_) => {}
    }

    let mut return_type = SnapshotChangeType::None;
    if !created.is_empty() { return_type = SnapshotChangeType::Created; }
    if !deleted.is_empty() { return_type = SnapshotChangeType::Deleted; }
    if !changed.is_empty() { return_type = SnapshotChangeType::Changed; }


    Some((return_type, SnapshotCompareResult {
        created,
        deleted,
        changed,
    }))
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
        let test_snap = Snapshot::new(Path::new("/etc"), HashType::BLAKE3);
        let stop = SystemTime::now();
        let lapsed = stop.duration_since(start).unwrap();
        println!("{:?}", lapsed);

        // let start2 = SystemTime::now();
        // // let test_snap = Snapshot::new(Path::new("/home/foxx/Downloads/"), HashType::Full);
        // let test_snap = Snapshot::new(Path::new("/bin"), HashType::Full);
        // let stop2 = SystemTime::now();
        // let lapsed2 = stop2.duration_since(start2).unwrap();
        // println!("{:?}", lapsed2);


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
