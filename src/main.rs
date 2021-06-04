use std::path::{Path, PathBuf};
use std::io::Write;
use std::fmt;

const SOURCE_DIR: &'static str = "/Users/z/Pictures/import";
const DEST_TOP: &'static str = "/Users/z/Pictures/sorted";

struct MoveInfo {
    source: PathBuf,
    dest: PathBuf
}
impl fmt::Display for MoveInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "s: {:?}, d: {:?}", self.source, self.dest)
    }
}

fn main() {
    // Create master list
    let photo_itr = std::fs::read_dir(Path::new(SOURCE_DIR)).expect("SOURCE_DIR itr is valid");
    let mut photo_itr_list = Vec::new();
    photo_itr_list.push(photo_itr);

    // Enter all directories and get files
    let photo_itr = std::fs::read_dir(Path::new(SOURCE_DIR)).expect("SOURCE_DIR itr is valid");
    let mut num_photos = 0;
    let mut num_subdir_photos = 0;
    let mut inc_func = || num_subdir_photos += 1;
    for item_res in photo_itr {
        let item = item_res.expect("Item is valid");
        let path = item.path();
        if path.is_dir() {
            // If directory, append directory itr to the vector
            visit_dirs(&path, &mut inc_func, &mut photo_itr_list).expect("Subdirs to be traversed");
        }
        else if path.is_file() {
            num_photos += 1;
        }
    }
    num_photos += num_subdir_photos;
    println!("Sorting {} photos", num_photos);

    let mut audit_list = Vec::new();
    let mut copy_list = Vec::new();
    for photo_res in photo_itr_list.into_iter().flatten() {
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
        println!("output from {:?} was: {}", photo.path(), stdout);
        if stdout.is_empty() {
            continue;
        }
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
        let relative_path = format!("{}/{}/{}", year, month, day);
        let path: PathBuf = [String::from(DEST_TOP), relative_path].iter().collect();
        // Check if file exists at location or in pending list
        let file_name = format!("{}-{}-{}_{}.{}.{}.{}",
                            year, month, day, hour, minute, second,
                            &photo.path().extension().expect("File has extension")
                                .to_str().expect("Can convert OsStr to str"));
        let mut file = path.to_path_buf();
        file.push(file_name);
        let file_exists = file.exists();
        let copy_info = MoveInfo{source: photo.path(), dest: file};
        if !file_exists {
            // Append item to list of things to move
            copy_list.push(copy_info);
        }
        else {
            // Don't clobber - append to a list of photos to audit
            audit_list.push(copy_info);
        }
    }

    // Confirm with user. Display a summary + get input
    let mut resp = String::new();
    let mut got_resp = false;
    while !got_resp {
        println!("About to move {} photos", &copy_list.len());
        if !audit_list.is_empty() {
            println!("There are {} files that need to be audited", &copy_list.len());
        }
        println!("Enter command:");
        println!("\t> more");
        if !audit_list.is_empty() {
            println!("\t> audit");
        }
        println!("\t> abort");
        println!("\t> confirm");
        get_resp(&mut resp);
        match resp.to_lowercase().as_str() {
            "more" => {
                println!("Listing all files to move:");
                for file in &copy_list {
                    println!("{}", file);
                }
            },
            "audit" => {
                println!("Listing all problem files:");
                for file in &audit_list {
                    println!("{}", file);
                }
            },
            "abort" => {
                println!("Aborting, not doing anything.");
                return;
            }
            "confirm" => {
                for file in &copy_list {
                    std::fs::create_dir_all(&file.dest.parent().expect("file.dest has parent")).expect("Can create directory");
                    match std::fs::copy(&file.source, &file.dest) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error copying: {:?} -> {:?}", &file.source, &file.dest);
                            println!("Error was: {}", e);
                        }
                    }
                    match std::fs::copy(&file.source.with_extension("MOV"), &file.dest.with_extension("MOV")) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error copying: {:?} -> {:?}", &file.source, &file.dest);
                            println!("Error was: {}", e);
                        }
                    }
                }
                got_resp = true;
            }
            _ => {
                println!("Invalid response: {}.", resp.to_lowercase().as_str());
            }
        }
    }

    // Ask to delete old files
    got_resp = false;
    while !got_resp {
        println!("Delete all old files?");
        println!("\t> yes");
        println!("\t> no");
        println!("\t> list");
        get_resp(&mut resp);
        match resp.to_lowercase().as_str() {
            "list" => {
                println!("Listing all files to delete:");
                for file in &copy_list {
                    println!("{}", file);
                }
            },
            "no" => {
                println!("Not deleting files.");
                got_resp = true;
            }
            "yes" => {
                print!("Deleting files... ");
                for file in &copy_list {
                    match std::fs::remove_file(&file.source) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error removing: {:?}", &file.source);
                            println!("Error was: {}", e);
                        }
                    }
                    match std::fs::remove_file(&file.source.with_extension("MOV")) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error removing: {:?}", &file.source);
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
        for file in &audit_list {
            println!("{}", file);
        }
    }
}

fn get_resp(resp: &mut String) {
    resp.clear();
    print!("(cmd): ");
    std::io::stdout().flush().expect("Can flush stdout");
    std::io::stdin().read_line(resp).expect("Read line works");
    resp.pop(); // Pop off newline
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(), list: &mut Vec<std::fs::ReadDir>) -> std::io::Result<()> {
    if dir.is_dir() {
        list.push(std::fs::read_dir(dir).expect("Sub-dir is valid"));
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb, list)?;
            } else {
                cb();
            }
        }
    }
    Ok(())
}
