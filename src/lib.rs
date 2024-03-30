extern crate core;

use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use anyhow::Error;
use crate::hasher::HashType;
use crate::snapshot::{Snapshot, SnapshotCompareResult, SnapshotChangeType, compare, export, import};
use thiserror::Error;
pub mod hasher;
pub mod snapshot;


pub fn create_snapshot(path: &str, hash_type: HashType, black_list: Vec<String>) -> Result<Snapshot, Error> {
    Ok(Snapshot::new(Path::new(path), hash_type, black_list)?)
}

pub fn compare_snapshots(left: Snapshot, right: Snapshot) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    compare(left, right)
}

pub fn export_snapshot(snapshot: Snapshot, path: String, overwrite: bool) -> Result<(), Error> {
    Ok(export(snapshot, path, overwrite)?)
}

pub fn import_snapshot(path: String) -> Result<Snapshot, Error> {
    Ok(import(path)?)
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
