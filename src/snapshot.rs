use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use rand::{Rng, thread_rng};
use crate::hasher::hash_file;

#[derive(Debug)]
pub struct Snapshot {
    pub file_hashes: HashMap<String, FileMetadata>,
    uuid: String
}
#[derive(Debug)]
pub struct FileMetadata {
    path: String,
    check_sum: Vec<u8>,
    size: usize,
}

impl Snapshot {
    fn new(path: &Path) -> Snapshot {
        let mut rand = thread_rng();
        let uuid_int: i128 = rand.gen();
        let uuid = uuid_int.to_string();

        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();

        let mut file_hashes: HashMap<String, FileMetadata> = HashMap::new();

        for path in file_paths {
            if let Ok(p) = path {
                if p.path().is_file() {
                    let (path, size, check_sum) = hash_file(p.path());
                    file_hashes.insert(p.path().to_str().unwrap().to_string(), FileMetadata {
                        path,
                        check_sum,
                        size,
                    });
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

        let test_snap = Snapshot::new(Path::new("/"));
        // let test_snap = Snapshot::new(Path::new("/home/foxx/Documents/pcidocs"));
        // println!("{}", test_snap.file_hashes.len());
        //

        println!("Files: {}", test_snap.file_hashes.len());

        // for fi in test_snap.file_hashes.iter() {
        //     println!("{}", fi.0)
        // }

        // assert_eq!(result, 4);
    }
}
