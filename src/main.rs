use std::path::Path;


const SOURCE_DIR: &'static str = "";
const DEST_TOP: &'static str = "";

fn main() {
    // Get how many new photos there are
    let num_photos = 69;
    println!("Sorting {} photos", num_photos);

    for photo in Path::new(SOURCE_DIR) {
        // Check exif data for photo date
        std::process::Command::new("heif-info");
        // Check if path exists, create if not
        // Check if file exists at location or in pending list
            // Don't clobber + append to a list of photos to audit
        // Append item to list of things to move
    }

    // Confirm with user. Display a summary + get input
    // More details (list all)
    // abort
    // confirm

    // Move all files to new area

    // Ask to delete old files
    // delete/abort
}
