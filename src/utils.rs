// Copyright ©️  Zach Nielsen 2021
use std::path::{Path, PathBuf};
use std::io::Write;
use std::fmt;

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
    let mut file_list = vec![source];
    let mov_file = source.with_extension("MOV");
    if source.extension().expect("File has extension") == "HEIC" {
        file_list.push(&mov_file);
    }

    for file in file_list {
        let result = match operation {
            PhotoOp::Copy => {
                match std::fs::copy(&file, &to.unwrap()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e)
                }
            }
            PhotoOp::Move => std::fs::rename(&file, &to.unwrap()),
            PhotoOp::Remove => std::fs::remove_file(&file),
        };
        match result {
            Ok(_) => (),
            Err(e) => {
                if let Some(opt_to) = to {
                    println!("Error {}: {:?} -> {:?}", operation, &file, &opt_to);
                }
                else {
                    println!("Error {}: {:?}", operation, &file);
                }
                println!("Error was: {}", e);
            }
        }
    }
}

pub fn get_photo_time(photo: &PathBuf) -> Option<String> {
    // TODO - MP4
    match get_exif_time(photo) {
        Some(out) => Some(out),
        None => {
            match photo.extension().expect("photo has extension").to_str() {
                Some("AAE") => {
                    // AAE is a slow motion sidecar file. Check the same basename for date.
                    get_base_photo_time(&photo)
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
    let basename = photo.file_stem().expect("file_stem exists")
                        .to_str().expect("Can convert OsStr to str");
    get_exif_time(format!("{}.HEIC", basename))
}

/// `photo` is assumed to have exif data. Use get_photo_time for files that don't have exif data.
pub fn get_exif_time(photo: &PathBuf) -> Option<String> {
    // identify -format "%[EXIF:DateTime]"
    let output = std::process::Command::new("identify")
                                        .arg("-format")
                                        .arg("%[EXIF:DateTime]")
                                        .arg(&photo)
                                        .output()
                                        .expect("Call to identify works");
    let stdout = String::from(std::str::from_utf8(&output.stdout).expect("stdout is stringable"));
    println!("output from {:?} was: {}", &photo, stdout);
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
