use crate::hasher::{hash_file, HashType};
use anyhow::{anyhow, Error};
use bytes::BytesMut;
use chrono::Utc;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{env, fs, thread};
use walkdir::DirEntry;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    pub black_list: Vec<String>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64,
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
    pub fn new(
        path: &Path,
        hash_type: HashType,
        black_list: Vec<String>,
    ) -> Result<Snapshot, Error> {
        let root_path = match path.to_str() {
            None => "".to_string(),
            Some(p) => p.to_string(),
        };
        let mut rand = thread_rng();
        let uuid_int: u128 = rand.gen();
        let uuid = uuid_int.to_string();
        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();
        let file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut pool: Vec<JoinHandle<()>> = vec![];

        let mut paths: Vec<Option<DirEntry>> = vec![];
        file_paths
            .into_iter()
            .flatten()
            .for_each(|a| paths.push(Option::from(a)));

        while !paths.is_empty() {
            #[allow(clippy::collapsible_match)]
            if let Some(p) = paths.pop() {
                if let Some(p) = p {
                    let mut blacklisted = false;
                    black_list.iter().for_each(|bl| {
                        if let Some(a) = p.path().to_str() {
                            if a.starts_with(bl) {
                                blacklisted = true
                            }
                        }
                    });

                    if p.path().is_file() && !blacklisted {
                        let bind = file_hashes.clone();

                        let handle = thread::spawn(move || {
                            let mut binding = bind.lock();
                            let ht = binding.as_mut().expect("binding error");
                            if let Err(e) = hash_file(p.path(), ht, hash_type) {
                                println!("Warning: {e}")
                            }
                        });
                        pool.push(handle);
                        if pool.len() > 4 {
                            if let Some(handle) = pool.pop() {
                                handle.join().expect("could not join handle")
                            }
                        }
                    }
                }
            }
        }

        for handle in pool {
            handle.join().expect("could not join handle")
        }

        Ok(Snapshot {
            file_hashes,
            black_list,
            root_path,
            hash_type,
            uuid,
            date_created: Utc::now().timestamp(),
        })
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        let black_list: Vec<String> = vec![];
        Snapshot {
            file_hashes: Arc::new(Mutex::new(HashMap::new())),
            black_list,
            root_path: "".to_string(),
            hash_type: HashType::BLAKE3,
            uuid: "".to_string(),
            date_created: 0,
        }
    }
}

#[derive(Debug)]
pub enum SnapshotChangeType {
    None,
    Created,
    Deleted,
    Changed,
}

#[derive(Debug)]
pub struct SnapshotCompareResult {
    pub created: Vec<String>,
    pub deleted: Vec<String>,
    pub changed: Vec<String>,
}

pub fn compare_hashes(
    left: Snapshot,
    right: Snapshot,
) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    #[allow(unused)]
    let success = true;
    let mut created: Vec<String> = vec![];
    let mut deleted: Vec<String> = vec![];
    let mut changed: Vec<String> = vec![];

    if let Ok(left_lock) = left.file_hashes.lock() {
        // for each entry in the hash list
        for left_entry in left_lock.iter() {
            if let Ok(curr_lock) = right.file_hashes.lock() {
                match curr_lock.get(left_entry.0) {
                    // check for mis-matching checksum between L and R
                    Some(right_entry) => {
                        if !right_entry.check_sum.eq(&left_entry.1.check_sum) {
                            changed.push(right_entry.path.to_string());
                        }
                    }
                    // check for deletion == files that exist in L and missing from R
                    None => {
                        deleted.push(left_entry.0.to_string());
                    }
                }
            }
        }
    }
    // check for creation == check for files that exist in R but do not exist in L
    if let Ok(e) = right.file_hashes.lock() {
        for right_entry in e.iter() {
            if left.file_hashes.lock().ok()?.get(right_entry.0).is_none() {
                created.push(right_entry.0.to_string());
            }
        }
    }

    let mut return_type = SnapshotChangeType::None;
    if !created.is_empty() {
        return_type = SnapshotChangeType::Created;
    }
    if !deleted.is_empty() {
        return_type = SnapshotChangeType::Deleted;
    }
    if !changed.is_empty() {
        return_type = SnapshotChangeType::Changed;
    }

    Some((
        return_type,
        SnapshotCompareResult {
            created,
            deleted,
            changed,
        },
    ))
}

pub fn compare_hashes_and_modify_date(
    left: Snapshot,
    right: Snapshot,
) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    #[allow(unused)]
    let success = true;
    let mut created: Vec<String> = vec![];
    let mut deleted: Vec<String> = vec![];
    let mut changed: Vec<String> = vec![];

    if let Ok(left_lock) = left.file_hashes.lock() {
        // for each entry in the hash list
        for left_entry in left_lock.iter() {
            if let Ok(curr_lock) = right.file_hashes.lock() {
                match curr_lock.get(left_entry.0) {
                    // check for mis-matching checksum between L and R
                    Some(right_entry) => {
                        // compare hashsum
                        if !right_entry.check_sum.eq(&left_entry.1.check_sum) {
                            changed.push(right_entry.path.to_string());
                        }
                        // compare modify date
                        if !right_entry.mtime.eq(&left_entry.1.mtime)
                            || !right_entry.ctime.eq(&left_entry.1.ctime)
                        {
                            changed.push(right_entry.path.to_string());
                        }
                    }
                    // check for deletion == files that exist in L and missing from R
                    None => {
                        deleted.push(left_entry.0.to_string());
                    }
                }
            }
        }
    }

    // check for creation == check for files that exist in R but do not exist in L
    if let Ok(e) = right.file_hashes.lock() {
        for right_entry in e.iter() {
            if left.file_hashes.lock().ok()?.get(right_entry.0).is_none() {
                created.push(right_entry.0.to_string());
            }
        }
    }

    let mut return_type = SnapshotChangeType::None;
    if !created.is_empty() {
        return_type = SnapshotChangeType::Created;
    }
    if !deleted.is_empty() {
        return_type = SnapshotChangeType::Deleted;
    }
    if !changed.is_empty() {
        return_type = SnapshotChangeType::Changed;
    }

    Some((
        return_type,
        SnapshotCompareResult {
            created,
            deleted,
            changed,
        },
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableSnapshot {
    pub file_hashes: Vec<FileMetadata>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64,
}

fn path_resolve(path: String) -> String {
    #[allow(unused)]
    let mut full_path = String::new();
    if path.starts_with("./") {
        let mut cur_dir: String = match env::current_dir() {
            Ok(pb) => match pb.to_str() {
                None => String::new(),
                Some(str) => str.to_string(),
            },
            Err(_) => String::new(),
        };
        cur_dir.push('/');
        full_path = path.replace("./", cur_dir.as_str());
    } else {
        full_path = path.to_string();
    }
    full_path
}

pub fn export(snapshot: Snapshot, path: String, overwrite: bool) -> Result<(), Error> {
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

    let serialized = serde_json::to_string(&serializable)?;
    // println!("{:#?}", serialized);
    let filename = full_path
        .split('/')
        .last()
        .expect("unable to get full path");
    let path_only = full_path.replace(filename, "");
    // println!("{}", full_path);
    // println!("{}", path_only);

    if Path::new(&full_path).exists() && overwrite {
        fs::remove_file(&full_path)?;
        write_to_file(path_only, full_path, serialized)?
    } else if !Path::new(&full_path).exists() {
        write_to_file(path_only, full_path, serialized)?
    };
    Ok(())
}

fn write_to_file(path_only: String, full_path: String, serialized: String) -> Result<(), Error> {
    if fs::create_dir_all(&path_only).is_ok() {
        if let Ok(mut file_handle) = File::create(full_path) {
            Ok(file_handle.write_all(serialized.as_bytes())?)
        } else {
            Err(anyhow!("Unable to write to path: {}", path_only.clone()))
        }
    } else {
        Err(anyhow!("Unable to write to path: {}", path_only.clone()))
    }
}

pub fn import(path: String) -> Result<Snapshot, Error> {
    #[allow(unused)]
    let buffer = BytesMut::new();
    let full_path = path_resolve(path);
    if let Ok(bytes) = fs::read(full_path) {
        let snapshot = serde_json::from_slice::<SerializableSnapshot>(&bytes)?;

        let mut fh: HashMap<String, FileMetadata> = HashMap::new();

        // println!("{}", snapshot.file_hashes.len());

        for entry in snapshot.file_hashes {
            if let Some(_res) = fh.insert(entry.path.clone(), entry.clone()) {
                // println!("successfully imported: {}", entry.path);
            }
        }
        let black_list: Vec<String> = vec![];
        Ok(Snapshot {
            file_hashes: Arc::new(Mutex::new(fh)),
            black_list,
            root_path: snapshot.root_path,
            hash_type: snapshot.hash_type,
            uuid: snapshot.uuid,
            date_created: snapshot.date_created,
        })
    } else {
        Ok(Snapshot::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compare_snapshots, compare_snapshots_including_modify_date};
    use std::fs;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn dangerous() {
        let mut snap = Snapshot::new(
            Path::new("/proc"),
            HashType::BLAKE3,
            vec![
                "testkey".to_string(),
                "/dev".to_string(),
                "/proc".to_string(),
                "/tmp".to_string(),
            ],
        );
        assert!(snap.is_ok());

        let mut snap = Snapshot::new(
            Path::new("/dev"),
            HashType::BLAKE3,
            vec![
                "testkey".to_string(),
                "/dev".to_string(),
                "/proc".to_string(),
                "/tmp".to_string(),
            ],
        );
        assert!(snap.is_ok());

        let mut snap = Snapshot::new(
            Path::new("/tmp"),
            HashType::BLAKE3,
            vec![
                "testkey".to_string(),
                "/dev".to_string(),
                "/proc".to_string(),
                "/tmp".to_string(),
            ],
        );
        assert!(snap.is_ok());
    }

    #[test]
    fn blacklist() {
        let mut snap = Snapshot::new(
            Path::new("/etc"),
            HashType::BLAKE3,
            vec!["testkey".to_string()],
        )
        .unwrap();
        println!("{:#?}", snap.clone().black_list);
        assert_eq!(snap.black_list.pop().unwrap(), "testkey".to_string());
    }

    #[test]
    fn create_snapshot_blake3() {
        let test_snap_b3 = Snapshot::new(Path::new("/etc"), HashType::BLAKE3, vec![]);
        assert!(test_snap_b3.unwrap().file_hashes.lock().unwrap().len() > 0);
    }
    #[test]
    fn create_snapshot_md5() {
        let test_snap_md5 = Snapshot::new(Path::new("/etc"), HashType::MD5, vec![]);
        assert!(test_snap_md5.unwrap().file_hashes.lock().unwrap().len() > 0);
    }
    #[test]
    fn create_snapshot_sha3() {
        let test_snap_sha3 = Snapshot::new(Path::new("/etc"), HashType::SHA3, vec![]);
        assert!(test_snap_sha3.unwrap().file_hashes.lock().unwrap().len() > 0);
    }

    #[test]
    fn export_snapshot() {
        assert!(!Path::new("./target/build/out.snapshot").exists());
        let test_snap_export = Snapshot::new(Path::new("/etc"), HashType::BLAKE3, vec![]);
        let _ = export(
            test_snap_export.unwrap().clone(),
            "./target/build/out.snapshot".to_string(),
            true,
        );
        assert!(Path::new("./target/build/out.snapshot").exists());
        fs::remove_file(Path::new("./target/build/out.snapshot")).unwrap();
    }

    #[test]
    fn import_snapshot() {
        let test_snap_import = Snapshot::new(Path::new("/etc"), HashType::BLAKE3, vec![]);
        let _ = export(
            test_snap_import.unwrap(),
            "./target/build/in.snapshot".to_string(),
            true,
        );
        let snapshot = import("./target/build/in.snapshot".to_string());
        assert!(snapshot.unwrap().file_hashes.lock().unwrap().len() > 0);
        fs::remove_file(Path::new("./target/build/in.snapshot")).unwrap();
    }

    #[test]
    fn creation_detection() {
        assert!(!Path::new("./target/build/test_creation/").exists());
        fs::create_dir_all(Path::new("./target/build/test_creation/")).unwrap();
        let test_snap_creation_1 = Snapshot::new(
            Path::new("./target/build/test_creation/"),
            HashType::BLAKE3,
            vec![],
        );
        File::create(Path::new("./target/build/test_creation/test1")).unwrap();
        File::create(Path::new("./target/build/test_creation/test2")).unwrap();
        File::create(Path::new("./target/build/test_creation/test3")).unwrap();
        let test_snap_creation_2 = Snapshot::new(
            Path::new("./target/build/test_creation/"),
            HashType::BLAKE3,
            vec![],
        );
        assert_eq!(
            compare_snapshots(test_snap_creation_1.unwrap(), test_snap_creation_2.unwrap())
                .unwrap()
                .1
                .created
                .len(),
            3
        );
        fs::remove_dir_all(Path::new("./target/build/test_creation/")).unwrap();
    }

    #[test]
    fn deletion_detection() {
        assert!(!Path::new("./target/build/test_deletion/").exists());
        fs::create_dir_all(Path::new("./target/build/test_deletion/")).unwrap();
        let test_snap_deletion_1 = Snapshot::new(
            Path::new("./target/build/test_deletion/"),
            HashType::BLAKE3,
            vec![],
        );
        File::create(Path::new("./target/build/test_deletion/test1")).unwrap();
        File::create(Path::new("./target/build/test_deletion/test2")).unwrap();
        File::create(Path::new("./target/build/test_deletion/test3")).unwrap();
        let test_snap_deletion_2 = Snapshot::new(
            Path::new("./target/build/test_deletion/"),
            HashType::BLAKE3,
            vec![],
        );
        assert_eq!(
            compare_snapshots(test_snap_deletion_2.unwrap(), test_snap_deletion_1.unwrap())
                .unwrap()
                .1
                .deleted
                .len(),
            3
        );
        fs::remove_dir_all(Path::new("./target/build/test_deletion/")).unwrap();
    }

    #[test]
    fn change_detection() {
        assert!(!Path::new("./target/build/test_change/").exists());
        fs::create_dir_all(Path::new("./target/build/test_change/")).unwrap();
        let mut file1 = File::create(Path::new("./target/build/test_change/test1")).unwrap();
        let mut file2 = File::create(Path::new("./target/build/test_change/test2")).unwrap();
        let mut file3 = File::create(Path::new("./target/build/test_change/test3")).unwrap();
        let test_snap_change_1 = Snapshot::new(
            Path::new("./target/build/test_change/"),
            HashType::BLAKE3,
            vec![],
        );
        file1.write_all("file1".as_bytes()).unwrap();
        file2.write_all("file2".as_bytes()).unwrap();
        file3.write_all("file3".as_bytes()).unwrap();
        let test_snap_change_2 = Snapshot::new(
            Path::new("./target/build/test_change/"),
            HashType::BLAKE3,
            vec![],
        );
        assert_eq!(
            compare_snapshots(test_snap_change_1.unwrap(), test_snap_change_2.unwrap())
                .unwrap()
                .1
                .changed
                .len(),
            3
        );
        fs::remove_dir_all(Path::new("./target/build/test_change/")).unwrap();
    }

    #[test]
    fn change_detection_including_modify_date() {
        assert!(!Path::new("./target/build/test_change_modify/").exists());
        fs::create_dir_all(Path::new("./target/build/test_change_modify/")).unwrap();
        let _ = File::create(Path::new("./target/build/test_change_modify/test1")).unwrap();
        let _ = File::create(Path::new("./target/build/test_change_modify/test2")).unwrap();
        let _ = File::create(Path::new("./target/build/test_change_modify/test3")).unwrap();
        let test_snap_change_1 = Snapshot::new(
            Path::new("./target/build/test_change_modify/"),
            HashType::BLAKE3,
            vec![],
        );
        let _ = fs::remove_file(Path::new("./target/build/test_change_modify/test1")).unwrap();
        let _ = fs::remove_file(Path::new("./target/build/test_change_modify/test2")).unwrap();
        let _ = fs::remove_file(Path::new("./target/build/test_change_modify/test3")).unwrap();

        let _ = File::create(Path::new("./target/build/test_change_modify/test1")).unwrap();
        let _ = File::create(Path::new("./target/build/test_change_modify/test2")).unwrap();
        let _ = File::create(Path::new("./target/build/test_change_modify/test3")).unwrap();
        let test_snap_change_2 = Snapshot::new(
            Path::new("./target/build/test_change_modify/"),
            HashType::BLAKE3,
            vec![],
        );
        assert_ne!(
            compare_snapshots_including_modify_date(
                test_snap_change_1.unwrap(),
                test_snap_change_2.unwrap()
            )
            .unwrap()
            .1
            .changed
            .len(),
            3
        );
        fs::remove_dir_all(Path::new("./target/build/test_change_modify/")).unwrap();
    }
}
