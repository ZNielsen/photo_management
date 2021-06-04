use std::path::{Path, PathBuf};
use std::fmt;

const SOURCE_DIR: &'static str = "/Users/z/Pictures/import";
const DEST_TOP: &'static str = "";

struct MoveInfo {
    source: PathBuf,
    dest: String
}
impl fmt::Display for MoveInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "s: {:?}, d: {}", self.source, self.dest)
    }
}

fn main() {
    // TODO - Get how many new photos there are
    let num_photos = 69;
    println!("Sorting {} photos", num_photos);

    let mut audit_list = Vec::new();
    let mut copy_list = Vec::new();
    let photo_itr = Path::new(SOURCE_DIR).read_dir().expect("Photo itr is valid");
    for photo_res in photo_itr {
        let photo = photo_res.expect("Photo is valid");
        // Check exif data for photo date
        // identify -format "%[EXIF:DateTime]"
        let output = std::process::Command::new("identify")
                                            .arg("-format")
                                            .arg("%[EXIF:DateTime]")
                                            .arg(photo.path())
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
        let file_name = format!("{}-{}-{}_{}:{}:{}.{:?}",
                            year, month, day, hour, minute, second,
                            Path::new(&photo.path()).extension().expect("File has extension"));
        let mut file = path.to_path_buf();
        file.push(file_name);
        let str_file = String::from(path.to_str().expect("Path is stringable"));
        let move_info = MoveInfo{source: photo.path(), dest: str_file};
        if !file.exists() {
            // Append item to list of things to move
            copy_list.push(move_info);
        }
        else {
            // Don't clobber + append to a list of photos to audit
            audit_list.push(move_info);
        }
    }

    // Confirm with user. Display a summary + get input
    let mut resp = String::new();
    let mut got_resp = false;
    while !got_resp {
        println!("About to move {} photos", copy_list.len());
        if audit_list.len() > 0 {
            println!("There are {} files that need to be audited", copy_list.len());
        }
        println!("Enter command:");
        println!("\tmore");
        println!("\taudit");
        println!("\tabort");
        println!("\tconfirm");
        print!("(cmd): ");
        let io = std::io::stdin().read_line(&mut resp).expect("Read line works");
        match resp.to_lowercase().as_str() {
            "more" => {
                println!("Listing all files to move:");
                for file in copy_list {
                    println!("{}", file);
                }
            },
            "audit" => {
                println!("Listing all problem files:");
                for file in audit_list {
                    println!("{}", file);
                }
            },
            "abort" => {
                println!("Aborting, not moving anything.");
                return;
            }
            "confirm" => {
                for file in copy_list {
                    match std::fs::copy(file.source, file.dest) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error copying: {:?} -> {}", file.source, file.dest);
                            println!("Error was: {}", e);
                        }
                    }
                }
                got_resp = true;
            }
            _ => {
                println!("Invalid response.");
            }
        }
    }

    // Ask to delete old files
    got_resp = false;
    while !got_resp {
        println!("Delete all old files?");
        println!("\tyes");
        println!("\tno");
        println!("\tlist");
        print!("(cmd): ");
        let io = std::io::stdin().read_line(&mut resp).expect("Read line works");
        match resp.to_lowercase().as_str() {
            "list" => {
                println!("Listing all files to delete:");
                for file in copy_list {
                    println!("{}", file);
                }
            },
            "no" => {
                println!("Not deleting files.");
                got_resp = true;
            }
            "yes" => {
                print!("Deleting files... ");
                for file in copy_list {

                    match std::fs::remove_file(file.source) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error removing: {:?}", file.source);
                            println!("Error was: {}", e);
                        }
                    }
                }
                println!("Done!");
                got_resp = true;
            }
            _ => {
                println!("Invalid response.");
            }
        }
    }

    if audit_list.len() > 0 {
        println!("There are {} files that need to be audited.", copy_list.len());
        println!("Listing all problem files:");
        for file in audit_list {
            println!("{}", file);
        }
    }
}
