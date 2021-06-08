## Requirements

Mac:
```
brew install imagemagick
```

## Usage
Export iPhone photos using Image Capture. Go into `src/main.rs` and change your source/destination, 
then `cargo run` to sort everything.
`identify` can be extremely slow on some files, especially large videos, so be patient if running
on a large library.

This project comes with no warrantee or guarantees of any kind -- use at your own risk.
You should 100% have a backup of everything this thing touches and hand verify every output before
deleting anything.
