use crate::hasher::{hash_files, HashType};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{env, fs, os, thread};
use std::fs::File;
use std::io::{Read, Write};
use std::thread::JoinHandle;
use std::time::SystemTime;
use bytes::BytesMut;
use chrono::Utc;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone)]
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    pub black_list: Vec<String>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64
}
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub fn new(path: &Path, hash_type: HashType, black_list: Vec<String>) -> Snapshot {
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
            let file_path = p.path().to_str().unwrap().to_string();
            if p.path().is_file() && !black_list.contains(&file_path) {
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

        Snapshot { file_hashes, black_list, root_path, hash_type, uuid, date_created: Utc::now().timestamp() }
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        let black_list: Vec<String> = vec![];
        Snapshot { file_hashes: Arc::new(Mutex::new(HashMap::new())), black_list, root_path: "".to_string(), hash_type: HashType::BLAKE3, uuid: "".to_string(), date_created: 0 }
    }
}

pub enum SnapshotChangeType {
    None,
    Created,
    Deleted,
    Changed
}

#[derive(Debug)]
pub struct SnapshotCompareResult {
    pub created: Vec<String>,
    pub deleted: Vec<String>,
    pub changed: Vec<String>
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
                        match curr_lock.get(left_entry.0) {
                            // check for mis-matching checksum
                            Some(right_entry) => {
                                if !right_entry.check_sum.eq(&left_entry.1.check_sum) {
                                    changed.push(right_entry.path.to_string());
                                }
                            }
                            // check for deletion
                            None => {deleted.push(left_entry.0.to_string());}
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableSnapshot {
    pub file_hashes: Vec<FileMetadata>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64
}


fn path_resolve(path: String) -> String {
    let mut full_path = String::new();
    if path.starts_with("./") {
        let mut cur_dir: String = match env::current_dir() {
            Ok(pb) => match pb.to_str() {
                None => String::new(),
                Some(str) => str.to_string()
            }
            Err(_) => String::new()
        };
        cur_dir.push('/');
        full_path = path.replace("./", cur_dir.as_str());
    } else {
        full_path = path.to_string();
    }
    full_path
}

pub fn export(snapshot: Snapshot, path: String, overwrite: bool) {
    let full_path = path_resolve(path);

    let mut fh: Vec<FileMetadata> = vec![];

    if let Ok(unlocked) = snapshot.file_hashes.lock() {
        for entry in unlocked.iter() {
            fh.push(entry.1.clone())
        }
    }

    let serializable = SerializableSnapshot {
        file_hashes: fh,
        root_path: snapshot.root_path,
        hash_type: snapshot.hash_type,
        uuid: snapshot.uuid,
        date_created: snapshot.date_created,
    };

    let serialized = serde_json::to_string(&serializable).unwrap();
    // println!("{:#?}", serialized);
    let filename = full_path.split('/').last().unwrap();
    let path_only = full_path.replace(filename, "");
    // println!("{}", full_path);
    // println!("{}", path_only);

    if Path::new(&full_path).exists() && overwrite {
        let _ = fs::remove_file(&full_path).unwrap();
        write_to_file(path_only, full_path, serialized);
    } else if !Path::new(&full_path).exists() {
        write_to_file(path_only, full_path, serialized);
    }
}

fn write_to_file(path_only: String, full_path: String, serialized: String) {
    if let Ok(_) = fs::create_dir_all(path_only) {
        if let Ok(mut file_handle) = File::create(full_path) {
            file_handle.write_all(serialized.as_bytes()).unwrap()
        }
    }
}

pub fn import(path: String) -> Snapshot {
    let mut buffer = BytesMut::new();
    let full_path = path_resolve(path);
    if let Ok(bytes) = fs::read(full_path) {
        let snapshot= serde_json::from_slice::<SerializableSnapshot>(&bytes).unwrap();

        let mut fh: HashMap<String, FileMetadata> = HashMap::new();

        // println!("{}", snapshot.file_hashes.len());

        for entry in snapshot.file_hashes {
            if let Some(_res) = fh.insert(entry.path.clone(), entry.clone()) {
                // println!("successfully imported: {}", entry.path);
            }
        }
        let black_list: Vec<String> = vec![];
        Snapshot {
            file_hashes: Arc::new(Mutex::new(fh)),
            black_list,
            root_path: snapshot.root_path,
            hash_type: snapshot.hash_type,
            uuid: snapshot.uuid,
            date_created: snapshot.date_created,
        }
    } else {
        Snapshot::default()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use super::*;
    use std::path::Path;
    use std::time::SystemTime;
    use crate::compare_snapshots;

    #[test]
    fn create_snapshot_blake3() {
        let test_snap_b3 = Snapshot::new(Path::new("/etc"), HashType::BLAKE3, vec![]);
        assert!(test_snap_b3.file_hashes.lock().unwrap().len() > 0);
    }
    #[test]
    fn create_snapshot_md5() {
        let test_snap_md5 = Snapshot::new(Path::new("/etc"), HashType::MD5, vec![]);
        assert!(test_snap_md5.file_hashes.lock().unwrap().len() > 0);
    }
    #[test]
    fn create_snapshot_sha3() {
        let test_snap_sha3 = Snapshot::new(Path::new("/etc"), HashType::SHA3, vec![]);
        assert!(test_snap_sha3.file_hashes.lock().unwrap().len() > 0);
    }

    #[test]
    fn export_snapshot() {
        assert!(!Path::new("./build/out.snapshot").exists());
        let test_snap_export = Snapshot::new(Path::new("/etc"), HashType::BLAKE3, vec![]);
        export(test_snap_export.clone(), "./build/out.snapshot".to_string(), true);
        assert!(Path::new("./build/out.snapshot").exists());
        fs::remove_file(Path::new("./build/out.snapshot")).unwrap();
    }

    #[test]
    fn import_snapshot() {
        let test_snap_import = Snapshot::new(Path::new("/etc"), HashType::BLAKE3, vec![]);
        export(test_snap_import.clone(), "./build/in.snapshot".to_string(), true);
        let snapshot = import("./build/in.snapshot".to_string());
        assert!(snapshot.file_hashes.lock().unwrap().len() > 0);
        fs::remove_file(Path::new("./build/in.snapshot")).unwrap();
    }

    #[test]
    fn creation_detection() {
        assert!(!Path::new("./build/test_creation/").exists());
        fs::create_dir_all(Path::new("./build/test_creation/")).unwrap();
        let test_snap_creation_1 = Snapshot::new(Path::new("./build/test_creation/"), HashType::BLAKE3, vec![]);
        File::create(Path::new("./build/test_creation/test1")).unwrap();
        File::create(Path::new("./build/test_creation/test2")).unwrap();
        File::create(Path::new("./build/test_creation/test3")).unwrap();
        let test_snap_creation_2 = Snapshot::new(Path::new("./build/test_creation/"), HashType::BLAKE3, vec![]);
        assert_eq!(compare_snapshots(test_snap_creation_1, test_snap_creation_2).unwrap().1.created.len(), 3);
        fs::remove_dir_all(Path::new("./build/test_creation/")).unwrap();
    }

    #[test]
    fn deletion_detection() {
        assert!(!Path::new("./build/test_deletion/").exists());
        fs::create_dir_all(Path::new("./build/test_deletion/")).unwrap();
        let test_snap_deletion_1 = Snapshot::new(Path::new("./build/test_deletion/"), HashType::BLAKE3, vec![]);
        File::create(Path::new("./build/test_deletion/test1")).unwrap();
        File::create(Path::new("./build/test_deletion/test2")).unwrap();
        File::create(Path::new("./build/test_deletion/test3")).unwrap();
        let test_snap_deletion_2 = Snapshot::new(Path::new("./build/test_deletion/"), HashType::BLAKE3, vec![]);
        assert_eq!(compare_snapshots(test_snap_deletion_2, test_snap_deletion_1).unwrap().1.deleted.len(), 3);
        fs::remove_dir_all(Path::new("./build/test_deletion/")).unwrap();
    }

    #[test]
    fn change_detection() {
        assert!(!Path::new("./build/test_change/").exists());
        fs::create_dir_all(Path::new("./build/test_change/")).unwrap();
        let mut file1 = File::create(Path::new("./build/test_change/test1")).unwrap();
        let mut file2 = File::create(Path::new("./build/test_change/test2")).unwrap();
        let mut file3 = File::create(Path::new("./build/test_change/test3")).unwrap();
        let test_snap_change_1 = Snapshot::new(Path::new("./build/test_change/"), HashType::BLAKE3, vec![]);
        file1.write_all("file1".as_bytes()).unwrap();
        file2.write_all("file2".as_bytes()).unwrap();
        file3.write_all("file3".as_bytes()).unwrap();
        let test_snap_change_2 = Snapshot::new(Path::new("./build/test_change/"), HashType::BLAKE3, vec![]);
        assert_eq!(compare_snapshots(test_snap_change_1, test_snap_change_2).unwrap().1.changed.len(), 3);
        fs::remove_dir_all(Path::new("./build/test_change/")).unwrap();
    }

}
