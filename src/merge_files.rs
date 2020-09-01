use super::{compute_hash, LineFormatter};
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
    formatter.writeln(out, "=".repeat(20))?;
    formatter.writeln(out, format!("merch::setup: <{}>", base_dir.display()))?;
    for (idx, path) in files.iter().enumerate() {
        if !path.is_file() {
            continue;
        }
        let mut file = File::open(path)?;
        let hash = compute_hash(&mut file)?;
        formatter.writeln(
            out,
            format!(
                "merch::existing-file: <{}> <{}> <{}>",
                path.display(),
                idx,
                hash
            ),
        )?;
    }

    formatter.writeln(out, "=".repeat(20))?;
    writeln!(out)?;

    for (idx, path) in files.iter().enumerate() {
        if !path.is_file() {
            continue;
        }
        let colon = if without_content { &"" } else { &":" };

        formatter.writeln(
            out,
            format!("merch::file: <{}> <{}>{}", path.display(), idx, colon),
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
