// Copyright ©️  Zach Nielsen 2021
use std::path::{Path, PathBuf};
use std::io::Write;
use std::fmt;

pub const TMP_DIR: &'static str = "/tmp/magick";

pub struct MoveInfo {
    pub source: PathBuf,
    pub dest: PathBuf
}
impl fmt::Display for MoveInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "s: {:?}, d: {:?}", self.source, self.dest)
    }
}

pub struct ExifTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8
}
impl ExifTime {
    pub fn parse(s: &str) -> ExifTime {
        let date_time: Vec<&str> = s.split(' ').collect();
        let date: Vec<&str> = date_time[0].split(':').collect();
        let time: Vec<&str> = date_time[1].split(':').collect();
        ExifTime {
            year: date[0].parse::<u16>().expect("Valid year"),
            month: date[1].parse::<u8>().expect("Valid month"),
            day: date[2].parse::<u8>().expect("Valid day"),
            hour: time[0].parse::<u8>().expect("Valid hour"),
            minute: time[1].parse::<u8>().expect("Valid minute"),
            second: time[2].parse::<u8>().expect("Valid second"),
        }
    }
    pub fn file_base_name(&self) -> String {
        format!("{}-{}-{}_{}.{}.{}",
            self.year, self.month, self.day,
            self.hour, self.minute, self.second)
    }
    pub fn relative_path(&self) -> String {
        format!("{}/{}/{}", self.year, self.month, self.day)
    }
}

#[derive(PartialEq)]
pub enum Mode {
    Copy,
    Move
}
impl Mode {
    pub fn other(&self) -> Mode {
        match self {
            Mode::Copy => Mode::Move,
            Mode::Move => Mode::Copy,
        }
    }
}
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Copy => write!(f, "copy"),
            Mode::Move => write!(f, "move"),
        }
    }
}

#[derive(PartialEq)]
pub enum PhotoOp {
    Copy,
    Move,
    Remove
}
impl fmt::Display for PhotoOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhotoOp::Copy => write!(f, "copy"),
            PhotoOp::Move => write!(f, "move"),
            PhotoOp::Remove => write!(f, "remove"),
        }
    }
}

pub fn operate_on_photo(operation: PhotoOp, source: &PathBuf, to: Option<&PathBuf>) {
    let result = match operation {
        PhotoOp::Copy => {
            match std::fs::copy(&source, &to.unwrap()) {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        }
        PhotoOp::Move => std::fs::rename(&source, &to.unwrap()),
        PhotoOp::Remove => std::fs::remove_file(&source),
    };
    match result {
        Ok(_) => (),
        Err(e) => {
            if let Some(opt_to) = to {
                println!("Error {}: {:?} -> {:?}", operation, &source, &opt_to);
            }
            else {
                println!("Error {}: {:?}", operation, &source);
            }
            println!("Error was: {}", e);
        }
    }
}

/// Will try to get time by proxy if this file does not have exif data
pub fn get_photo_time(photo: &PathBuf) -> Option<String> {
    // TODO - MP4
    match get_exif_time(photo) {
        Some(out) => Some(out),
        None => {
            let extension = match photo.extension() {
                Some(val) => val,
                None => return None,
            };
            match extension.to_str() {
                Some("AAE") | Some("MOV") => {
                    // AAE is a slow motion sidecar file.
                    // MOV typically has a partner file with exif data, but not always.
                    // Check the same basename for date.
                    return get_base_photo_time(&photo)
                }
                Some(_) | None => {
                    println!("***** EMPTY OUTPUT, SKIPPING FILE *****");
                    return None;
                }
            }
        }
    }
}

pub fn get_base_photo_time(photo: &PathBuf) -> Option<String> {
    let photo_basename = String::from(photo.file_stem().expect("photo has stem").to_str().unwrap());
    let photo_ext = String::from(photo.extension().expect("photo has extension").to_str().unwrap());
    let mut match_list = Vec::new();
    let containing_dir = photo.parent().expect("photo has parent");
    for maybe_file in containing_dir.read_dir().expect("Can read dir") {
        let file = maybe_file.expect("A valid file").path();
        let file_basename = String::from(file.file_stem().expect("file has stem").to_str().unwrap());
        let file_ext = String::from(file.extension().unwrap_or(&std::ffi::OsString::from(&photo_ext)).to_str().unwrap());
        if file_basename == photo_basename && file_ext != photo_ext {
            match_list.push(file);
        }
    };
    println!("match_list size: {}", match_list.len());
    for file in match_list {
        match get_exif_time(&file) {
            Some(out) => return Some(out),
            None => (),
        }
    }
    None
}

/// Just gets exif data for this file. Use get_photo_time to also try getting time by proxy.
pub fn get_exif_time(photo: &PathBuf) -> Option<String> {
    static mut COUNTER: u64 = 0;
    std::fs::create_dir_all(&TMP_DIR).expect("Can make tmp dir");
    // MAGICK_TEMPORARY_PATH=/tmp/magick identify -format "%[EXIF:DateTime]"
    let output = std::process::Command::new("identify")
                                        .arg("-format")
                                        .arg("%[EXIF:DateTime]")
                                        .arg(&photo)
                                        .env("MAGICK_TEMPORARY_PATH", &TMP_DIR)
                                        .output()
                                        .expect("Call to identify works");
    let stdout = String::from(std::str::from_utf8(&output.stdout).expect("stdout is stringable"));
    println!("output from {:?} was: {}", &photo, stdout);
    unsafe {
        COUNTER += 1;
        if COUNTER % 10 == 0 {
            std::fs::remove_dir_all(&TMP_DIR).expect("Can remove tmp dir");
        }
    }
    if stdout.is_empty() {
        None
    }
    else {
        Some(stdout)
    }
}

pub fn get_resp(resp: &mut String) {
    resp.clear();
    print!("(cmd): ");
    std::io::stdout().flush().expect("Can flush stdout");
    std::io::stdin().read_line(resp).expect("Read line works");
    resp.pop(); // Pop off newline
}

pub fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(), list: &mut Vec<std::fs::ReadDir>) -> std::io::Result<()> {
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
