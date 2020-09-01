use blake2::digest::VariableOutput;
use blake2::VarBlake2b;
use glob::glob;
use pathdiff::diff_paths;
use std::io::{copy, Error, Read, Write};
use std::path::PathBuf;
use std::process::Command;

pub fn compute_hash(read: &mut impl Read) -> Result<String, Error> {
    let mut hasher = VarBlake2b::new(10).unwrap();
    copy(read, &mut hasher)?;
    let hash = hex::encode(hasher.vec_result());
    Ok(hash)
}

pub struct LineFormatter {
    pub prefix: String,
    pub suffix: String,
}

impl LineFormatter {
    pub fn writeln<W: Write>(&self, writer: &mut W, text: String) -> Result<(), std::io::Error> {
        writeln!(writer, "{}{}{}", self.prefix, text, self.suffix)
    }
}

pub fn normalize_glob(globs: Vec<PathBuf>, base_dir: PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for file in globs {
        for entry in glob(file.to_str().unwrap()).unwrap() {
            match entry {
                Ok(path) => {
                    let path =
                        diff_paths(&std::fs::canonicalize(path).unwrap(), &base_dir).unwrap();

                    paths.push(path.clone());
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }
    paths
}

pub fn launch_default_editor(file: PathBuf) -> Result<(), ()> {
    let output = Command::new("git")
        .args(&["config", "--get", "core.editor"])
        .output()
        .expect("You must have git installed");
    let result = String::from_utf8(output.stdout).expect("Output must be unicode.");

    let parts = shlex::split(&result).unwrap();
    let result = Command::new(&parts[0])
        .args(&parts[1..])
        .arg(&file)
        .status()
        .expect("Editor should launch");

    if result.success() {
        Ok(())
    } else {
        Err(())
    }
}
