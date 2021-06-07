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
