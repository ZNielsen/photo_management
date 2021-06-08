// Copyright ©️  Zach Nielsen 2021
mod utils;

use std::path::{Path, PathBuf};
use utils::*;

const SOURCE_DIR: &'static str = "/Users/z/Pictures/import";
const DEST_TOP: &'static str = "/Users/z/Pictures/sorted";

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

    let mut sorted_vec = photo_itr_list.into_iter().flatten()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .expect("Photos are valid and mapped");
    sorted_vec.sort();
    for photo in sorted_vec {
        // Check exif data for photo date
        let exif_time = match get_photo_time(&photo) {
            Some(out) => ExifTime::parse(&out),
            None => continue,
        };

        // Check if path exists, create if not
        let path: PathBuf = [String::from(DEST_TOP), exif_time.relative_path()].iter().collect();
        // Check if file exists at location or in pending list
        let file_name = format!("{}.{}", exif_time.file_base_name(),
                            &photo.extension().expect("File has extension")
                                .to_str().expect("Can convert OsStr to str"));
        let mut file = path.to_path_buf();
        file.push(file_name);
        let file_exists = file.exists();
        let copy_info = MoveInfo{source: photo, dest: file};
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
    let mut mode = Mode::Copy;
    while !got_resp {
        println!("");
        println!("About to {} {} photos", mode, &copy_list.len());
        if !audit_list.is_empty() {
            println!("There are {} files that need to be audited", &copy_list.len());
        }
        println!("Enter command:");
        println!("\t> more");
        if !audit_list.is_empty() {
            println!("\t> audit");
        }
        println!("\t> {}", mode.other());
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
            },
            "move" => {
                println!("Switching to Move mode.");
                mode = Mode::Move;
            }
            "confirm" => {
                for file in &copy_list {
                    std::fs::create_dir_all(&file.dest.parent().expect("file.dest has parent")).expect("Can create directory");
                    if mode == Mode::Copy {
                        operate_on_photo(PhotoOp::Copy, &file.source, Some(&file.dest));
                    }
                    else {
                        operate_on_photo(PhotoOp::Move, &file.source, Some(&file.dest));
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
    if mode == Mode::Copy {
        got_resp = false;
        while !got_resp {
            println!("");
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
                        operate_on_photo(PhotoOp::Remove, &file.source, None);
                    }
                    println!("Done!");
                    got_resp = true;
                }
                _ => {
                    println!("Invalid response.");
                }
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

