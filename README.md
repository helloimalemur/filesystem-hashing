#### filesystem-hashing

## Track Filesystem Integrity via "Snapshots"
    ~ Snapshots contain a HashMap containting all the files within specified directories,
    and their corresponding hash signature.
    ~ Snapshots are exported as JSON files.

## Snapshot structure
```rust
pub struct Snapshot {
    pub file_hashes: Arc<Mutex<HashMap<String, FileMetadata>>>,
    pub black_list: Vec<String>,
    pub root_path: String,
    pub hash_type: HashType,
    pub uuid: String,
    pub date_created: i64,
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
