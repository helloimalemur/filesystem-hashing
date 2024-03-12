#### Fasching

## Track Filesystem Integrity via "Snapshots" containing md5 or sha3 hash of all files within specified directories.
    Create Snapshots    
    Compare Snapshots

## Notes
    It is good practice to exclude tmp directories, mail spools, log directories, proc filesystems,
    user's home directories, web content directories, anything that changes regularly.
    It is also good practice to include all system binaries, libraries, include files, system source files.
    It is advisable to also include directories you don't often look in such as /dev, or /usr/man/.
    Of course you'll want to include as many files as practical, but think about what you include.

## Development and Collaboration
#### Feel free to open a pull request, please run the following prior to your submission please!
    echo "Run clippy"; cargo clippy -- -D clippy::all
    echo "Format source code"; cargo fmt -- --check
