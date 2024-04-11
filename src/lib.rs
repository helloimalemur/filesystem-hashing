#![allow(non_snake_case)]
extern crate core;

use crate::hasher::HashType;
use crate::snapshot::{
    compare, export, import, Snapshot, SnapshotChangeType, SnapshotCompareResult,
};
use anyhow::Error;
use std::path::Path;
pub mod hasher;
pub mod snapshot;

pub fn create_snapshot(
    path: &str,
    hash_type: HashType,
    black_list: Vec<String>,
) -> Result<Snapshot, Error> {
    Snapshot::new(Path::new(path), hash_type, black_list)
}

pub fn compare_snapshots(
    left: Snapshot,
    right: Snapshot,
) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    compare(left, right)
}

pub fn export_snapshot(snapshot: Snapshot, path: String, overwrite: bool) -> Result<(), Error> {
    export(snapshot, path, overwrite)
}

pub fn import_snapshot(path: String) -> Result<Snapshot, Error> {
    import(path)
}
