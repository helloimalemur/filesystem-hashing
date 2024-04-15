#### filesystem-hashing

## Track Filesystem Integrity via Snapshots
    ~ contain a HashMap of the files and their corresponding hash signature from a specified directory.
    ~ are exported as JSON files.

## Snapshot structure
```rust

pub enum HashType {
    MD5,
    SHA3,
    BLAKE3,
}
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    pub black_list: Vec<String>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64,
}
pub struct FileMetadata {
    pub path: String,
    pub check_sum: Vec<u8>,
    pub size: u64,
    pub ino: u64,
    pub ctime: i64,
    pub mtime: i64,
}
```
### Snapshot Comparison result structure
```rust
    pub enum SnapshotChangeType {
        None,
        Created,
        Deleted,
        Changed,
    }
    pub struct SnapshotCompareResult {
        pub created: Vec<String>,
        pub deleted: Vec<String>,
        pub changed: Vec<String>,
    }

```

## Usage
```rust
fn main() {
    /// snapshot creation
    let snapshot = create_snapshot("/etc", BLAKE3, vec![])?;
    let snapshot2 = create_snapshot("/etc", BLAKE3, vec![])?;
    
    /// snapshot export
    export_snapshot(snapshot.clone(), "./".to_string(), true)?;
    
    /// compare snapshots
    let results: (SnapshotChangeType, SnapshotCompareResult) = compare_snapshots(snapshot(), snapshot2)?;
}
```



## Utilized in the following project(s)
#### [sys-compare](https://github.com/helloimalemur/sys-compare)

## Notes
    ~ It is advised to **exclude** tmp directories, mail spools, log directories, proc filesystems,
    user's home directories, web content directories, and psuedo-device files.
    ~ It is advised to **include** all system binaries, libraries, include files, system source files.
    ~ It is also advisable to include directories you don't often look in such as /dev, or /usr/man/.

## Development and Collaboration
#### Feel free to open a pull request, please run the following prior to your submission please!
    echo "Run clippy"; cargo clippy -- -D clippy::all
    echo "Format source code"; cargo fmt -- --check
