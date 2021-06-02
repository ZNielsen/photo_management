use std::path::Path;

const SOURCE_DIR: &'static str = "/Users/z/Pictures/import";
const DEST_TOP: &'static str = "";

fn main() {
    // Get how many new photos there are
    let num_photos = 69;
    println!("Sorting {} photos", num_photos);

    let mut audit_list = Vec::new();
    let mut move_list = Vec::new();
    for photo in Path::new(SOURCE_DIR) {
        // Check exif data for photo date
        let output = std::process::Command::new("heif-info")
                                            .arg(photo)
                                            .output()
                                            .expect("heif-info failed");
        println!("output was: {:?}", output.stdout);
        // TODO - parse date
        let year = 0;
        let month = 0;
        let day = 0;
        let hour = 0;
        let minute = 0;
        let second = 0;

        // Check if path exists, create if not
        let path: Path = [SOURCE_DIR, format!("{}/{}/{}", year, month, day)].iter().collect();
        if !path.is_dir() {
            std::fs::create_dir_all(&path);
        }
        // Check if file exists at location or in pending list
        let file_name = format!("{}_{}_{}_{}_{}_{}.{:?}",
                            year, month, day, hour, minute, second,
                            Path::new(photo).extension().expect("File has extension"));
        let mut file = path.to_path_buf();
        file.push(file_name);
        let str_file = String::from(path.to_str().expect("Path is stringable"));
        if !file.exists() {
            // Append item to list of things to move
            move_list.push(str_file);
        }
        else {
            // Don't clobber + append to a list of photos to audit
            audit_list.push(str_file);
        }
    }


    // Confirm with user. Display a summary + get input
    // More details (list all)
    // abort
    // confirm

    // Move all files to new area

    // Ask to delete old files
    // delete/abort
}
