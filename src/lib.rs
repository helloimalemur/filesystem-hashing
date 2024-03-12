use std::path::Path;
use crate::hasher::HashType;
use crate::snapshot::Snapshot;

pub mod hasher;
pub mod snapshot;

pub fn create_snapshot(path: &str, hash_type: HashType) -> Snapshot {
    Snapshot::new(Path::new("/etc"), hash_type)
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
