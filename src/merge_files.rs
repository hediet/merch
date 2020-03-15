use super::{compute_hash, LineFormatter};
use glob::glob;
use pathdiff::diff_paths;
use std::fs::File;
use std::io::BufReader;
use std::io::Error;
use std::io::{copy, Write};
use std::path::PathBuf;

pub fn merge_files<W: Write>(
    files: Vec<PathBuf>,
    out: &mut W,
    formatter: &LineFormatter,
    base_dir: PathBuf,
    without_content: bool,
) -> Result<(), Error> {
    let mut paths = Vec::new();

    for file in files {
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

    formatter.writeln(out, "=".repeat(20))?;
    formatter.writeln(out, format!("merch::setup: <{}>", base_dir.display()))?;

    for (idx, path) in paths.iter().enumerate() {
        let mut file = File::open(path)?;
        let hash = compute_hash(&mut file)?;
        formatter.writeln(
            out,
            format!(
                "merch::existing-file: <{}> <{}> <{}>",
                idx,
                path.display(),
                hash
            ),
        )?;
    }

    formatter.writeln(out, "=".repeat(20))?;
    writeln!(out)?;

    for (idx, path) in paths.iter().enumerate() {
        let colon = if without_content { &"" } else { &":" };

        formatter.writeln(
            out,
            format!("merch::file: <{}> <{}>{}", idx, path.display(), colon),
        )?;

        if !without_content {
            let f = File::open(path)?;
            let mut file = BufReader::new(&f);
            copy(&mut file, out)?;
            writeln!(out)?;
        }
    }

    Ok(())
}
