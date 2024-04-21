#![allow(non_snake_case)]
extern crate core;

use crate::hasher::HashType;
use crate::snapshot::{
    compare_hashes, export, import, Snapshot, SnapshotChangeType, SnapshotCompareResult,
};
use anyhow::Error;
use std::path::Path;
pub mod hasher;
pub mod snapshot;

pub fn create_snapshot(
    path: &str,
    hash_type: HashType,
    black_list: Vec<String>,
    verbose: bool
) -> Result<Snapshot, Error> {
    Snapshot::new(Path::new(path), hash_type, black_list, verbose)
}

pub fn compare_snapshots(
    left: Snapshot,
    right: Snapshot,
    verbose: bool
) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    compare_hashes(left, right, verbose)
}

pub fn compare_snapshots_including_modify_date(
    left: Snapshot,
    right: Snapshot,
    verbose: bool
) -> Option<(SnapshotChangeType, SnapshotCompareResult)> {
    compare_hashes(left, right, verbose)
}

pub fn export_snapshot(snapshot: Snapshot, path: String, overwrite: bool, verbose: bool) -> Result<(), Error> {
    export(snapshot, path, overwrite, verbose)
}

pub fn import_snapshot(path: String, verbose: bool) -> Result<Snapshot, Error> {
    import(path, verbose)
}
