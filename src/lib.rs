extern crate core;

use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::hasher::HashType;
use crate::snapshot::{Snapshot, SnapshotCompareResult, SnapshotChangeType, compare, export, import};

pub mod hasher;
pub mod snapshot;

pub fn create_snapshot(path: &str, hash_type: HashType, black_list: Vec<String>) -> Snapshot {
    Snapshot::new(Path::new(path), hash_type, black_list)
}

pub fn compare_snapshots(left: Snapshot, right: Snapshot) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    compare(left, right)
}

pub fn export_snapshot(snapshot: Snapshot, path: String) {
    export(snapshot, path)
}

pub fn import_snapshot(path: String) -> Snapshot {
    import(path)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
