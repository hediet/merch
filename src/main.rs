use std::ffi::OsString;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::PathBuf;
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
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>,

        #[structopt(short = "m", long = "merch-file", parse(from_os_str))]
        out: Option<PathBuf>,

        #[structopt(
            short = "c",
            long = "comment-style",
            parse(from_os_str),
            default_value = "// {}"
        )]
        comment_style: OsString,
    },

    #[structopt(name = "split")]
    Split {
        #[structopt(short = "m", long = "merch-file", parse(from_os_str))]
        file: PathBuf,
    },
}

fn main() {
    let cmd = CliCommand::from_args();

    match cmd {
        CliCommand::Merge {
            files,
            out,
            comment_style,
        } => {
            let comment_style = comment_style.to_str().unwrap();
            let parts: Vec<_> = comment_style.split("{}").collect();

            let mut out: Box<dyn Write> = match out {
                Some(path_buf) => Box::new(File::create(path_buf).unwrap()),
                None => Box::new(stdout()),
            };

            let formatter = LineFormatter {
                prefix: parts[0].to_owned(),
                suffix: parts[1].to_owned(),
            };
            let current_dir = std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap();
            merge_files::merge_files(files, &mut out, &formatter, current_dir).unwrap();
        }

        CliCommand::Split { file } => {
            let mut file = File::open(file).unwrap();
            split_file::split_file(&mut file).unwrap();
        }
    }
}
