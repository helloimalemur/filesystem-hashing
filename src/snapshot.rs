use std::collections::HashMap;
use crate::file::FileMetadata;
use crate::file_hash::hash_file;

struct Snapshot {
    file_hashes: HashMap<String, FileMetadata>,
    uuid: String
}


impl Snapshot {
    fn new(path: String) -> Snapshot {
        let file_paths = walkdir::WalkDir::new(path).sort_by_file_name();

        for path in file_paths {
            if let Ok(p) = path {
                if p.path().is_file() {
                    hash_file(p.path())
                }
            }
        }

        let file_hashes: HashMap<String, FileMetadata> = HashMap::new();

        Snapshot { file_hashes: Default::default(), uuid: "".to_string() }
    }
}
