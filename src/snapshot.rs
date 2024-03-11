use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::str::Bytes;
use bytes::Bytes;
use crate::file_hash::hash_file;

struct Snapshot {
    file_hashes: HashMap<String, FileMetadata>,
    uuid: String
}

pub struct FileMetadata {
    path: String,
    check_sum: Bytes,
    size: u128,
}

impl Snapshot {
    fn new(path: &Path) -> Snapshot {
        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();

        let mut file_hashes: HashMap<String, FileMetadata> = HashMap::new();

        for path in file_paths {
            if let Ok(p) = path {
                if p.path().is_file() {
                    let hash_bytes: Bytes = hash_file(p.path());
                }
            }
        }



        Snapshot { file_hashes: Default::default(), uuid: "".to_string() }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn create_snapshot() {

        let test_snap = Snapshot::new(Path::new("./"));

        // assert_eq!(result, 4);
    }
}
