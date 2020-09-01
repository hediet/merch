use crate::utils::launch_default_editor;
use crate::utils::normalize_glob;
use std::ffi::OsString;
use std::fs::File;
use std::io::{prelude::*, stdin, stdout, BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

mod merge_files;
mod split_file;
mod utils;

pub use utils::{compute_hash, LineFormatter};

#[derive(StructOpt, Debug)]
#[structopt(name = "merch")]
pub enum CliCommand {
    #[structopt(name = "merge")]
    Merge {
        #[structopt(flatten)]
        common: CommonFlags,

        #[structopt(short = "m", long = "merch-file", parse(from_os_str))]
        out: Option<PathBuf>,
    },

    #[structopt(name = "edit")]
    Edit {
        #[structopt(flatten)]
        common: CommonFlags,
    },

    #[structopt(name = "split")]
    Split {
        #[structopt(short = "m", long = "merch-file", parse(from_os_str))]
        file: PathBuf,
        #[structopt(short = "d", long = "dry")]
        dry: bool,
    },
}

#[derive(StructOpt, Debug)]
pub struct CommonFlags {
    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>,

    #[structopt(short = "i", long = "input-file", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(
        short = "c",
        long = "comment-style",
        parse(from_os_str),
        default_value = "// {}"
    )]
    comment_style: OsString,

    #[structopt(short = "-w", long = "without-content")]
    without_content: bool,
}

fn main() {
    let cmd = CliCommand::from_args();

    match cmd {
        CliCommand::Edit { common } => {
            let ProcessedArgs {
                files,
                formatter,
                current_dir,
                without_content,
            } = process_common(common);
            let suffix = if let Some(first) = files.first() {
                if let Some(ext) = first.extension() {
                    ".".to_owned() + ext.to_str().unwrap()
                } else {
                    ".txt".to_owned()
                }
            } else {
                ".txt".to_owned()
            };

            let temp_file = tempfile::Builder::new().suffix(&suffix).tempfile().unwrap();

            let mut out = Box::new(temp_file.as_file());
            merge_files::merge_files(files, &mut out, &formatter, current_dir, without_content)
                .unwrap();

            let result = launch_default_editor(temp_file.path().to_path_buf());

            if result.is_err() {
                eprintln!("aborted");
                std::process::exit(1);
            }

            out.seek(std::io::SeekFrom::Start(0)).unwrap();
            split_file::split_file(&mut out, false).unwrap();
        }

        CliCommand::Merge { common, out } => {
            let ProcessedArgs {
                files,
                formatter,
                current_dir,
                without_content,
            } = process_common(common);

            let mut out: Box<dyn Write> = match out {
                Some(path_buf) => Box::new(File::create(path_buf).unwrap()),
                None => Box::new(stdout()),
            };

            merge_files::merge_files(files, &mut out, &formatter, current_dir, without_content)
                .unwrap();
        }

        CliCommand::Split { file, dry } => {
            let mut file = File::open(file).unwrap();
            split_file::split_file(&mut file, dry).unwrap();
        }
    }
}

struct ProcessedArgs {
    current_dir: PathBuf,
    formatter: LineFormatter,
    files: Vec<PathBuf>,
    without_content: bool,
}

fn process_common(common: CommonFlags) -> ProcessedArgs {
    let CommonFlags {
        mut files,
        input,
        comment_style,
        without_content,
    } = common;

    let comment_style = comment_style.to_str().unwrap();
    let parts: Vec<_> = comment_style.split("{}").collect();
    let formatter = LineFormatter {
        prefix: parts[0].to_owned(),
        suffix: parts[1].to_owned(),
    };

    if let Some(input) = input {
        let input: Box<dyn Read> = if input.to_str().unwrap() == "-" {
            Box::new(stdin())
        } else {
            Box::new(File::open(input).unwrap())
        };

        let reader = BufReader::new(input);
        for line in reader.lines() {
            files.push(PathBuf::from_str(&line.unwrap()).unwrap());
        }
    }

    let current_dir = std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap();

    ProcessedArgs {
        current_dir: current_dir.clone(),
        formatter,
        files: normalize_glob(files, current_dir.clone()),
        without_content,
    }
}
