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
        // identify -format "%[EXIF:DateTime]"
        let output = std::process::Command::new("identify")
                                            .arg("-format")
                                            .arg("%[EXIF:DateTime]")
                                            .arg(photo)
                                            .output()
                                            .expect("Get EXIF data");
        let stdout = String::from(std::str::from_utf8(&output.stdout).expect("stdout is stringable"));
        println!("output was: {}", stdout);
        let date_time: Vec<&str> = stdout.split(' ').collect();
        let date: Vec<&str> = date_time[0].split(':').collect();
        let time: Vec<&str> = date_time[1].split(':').collect();
        let year = date[0];
        let month = date[1];
        let day = date[2];
        let hour = time[0];
        let minute = time[1];
        let second = time[2];

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
    let mut resp = String::new();
    let io = std::io::stdin().read_line(&mut resp).expect("Readline works");
    let mut got_resp = false;
    while !got_resp {
        println!("TODO - Print the info + options");
        match resp.to_lowercase().as_str() {
            "more" => {
                // List all files to move
            },
            "abort" => {
            }
            "confirm" => {
                got_resp = true;
            }
            _ => {
                println!("Invalid response");
            }
        }
    }
    // More details (list all)
    // abort
    // confirm

    // Move all files to new area
    // Ask to delete old files

    // delete/abort
}
