use super::{compute_hash, LineFormatter};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;

#[derive(Debug)]
struct ExistingFileInfo {
    idx: usize,
    path: PathBuf,
    hash: String,
}

#[derive(Debug)]
struct UpdatedFileInfo<'t> {
    idx: usize,
    new_path: PathBuf,
    content: Option<&'t str>,
}

pub fn split_file(merch_file: PathBuf, formatter: &LineFormatter) -> Result<(), Error> {
    let mut file = File::open(merch_file).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let merch_instruction_regex = regex::escape(&formatter.prefix)
        + "merch::(?P<instruction>.*)"
        + &regex::escape(&formatter.suffix)
        + "\r?\n";
    let merch_instruction_regex = Regex::new(&merch_instruction_regex).unwrap();

    let instruction_regex =
        Regex::new(&"(?P<command>[a-zA-Z0-9-]+):\\s*(?P<args>(<.*?>\\s*)*)").unwrap();
    let arg_regex = Regex::new(&"<(?P<arg>.*?)>").unwrap();

    let mut existing_files: Vec<ExistingFileInfo> = Vec::new();
    let mut updated_files: HashMap<usize, UpdatedFileInfo> = HashMap::new();

    let mut last_file_idx = None;
    let mut last_full_match: Option<regex::Match> = None;
    for instruction_match in merch_instruction_regex
        .captures_iter(&content)
        .map(|m| Some(m))
        .chain(vec![None])
    {
        if let Some(last_match) = last_full_match {
            if let Some(last_file_idx) = last_file_idx {
                let mut end = if let Some(ref instruction_match) = instruction_match {
                    let full_match = instruction_match.get(0).unwrap();
                    full_match.start()
                } else {
                    content.len()
                };
                if &content[end - 1..end] == "\n" {
                    end -= 1;
                }
                if &content[end - 1..end] == "\r" {
                    end -= 1;
                }

                let text_in_between = &content[last_match.end()..end];
                let file_info = updated_files.get_mut(&last_file_idx).unwrap();
                file_info.content = Some(text_in_between);
            }
        }

        if let Some(ref instruction_match) = instruction_match {
            let full_match = instruction_match.get(0).unwrap();
            last_full_match = Some(full_match.clone());

            let instruction = instruction_match.get(1).unwrap().as_str();
            let m = instruction_regex.captures(&instruction).unwrap();
            let command = m.name(&"command").unwrap().as_str();
            let args = m.name(&"args").unwrap().as_str();
            let args: Vec<_> = arg_regex
                .captures_iter(&args)
                .map(|c| c.name(&"arg").unwrap().as_str())
                .collect();

            last_file_idx = None;
            match command {
                "existing-file" => {
                    let idx: usize = args[0].parse().unwrap();
                    let path: PathBuf = args[1].parse().unwrap();
                    let hash = args[2].to_owned();

                    existing_files.push(ExistingFileInfo { idx, path, hash });
                }
                "file" => {
                    let idx: usize = args[0].parse().unwrap();
                    let new_path: PathBuf = args[1].parse().unwrap();

                    last_file_idx = Some(idx);
                    updated_files.insert(
                        idx,
                        UpdatedFileInfo {
                            idx,
                            new_path,
                            content: None,
                        },
                    );
                }
                _ => {}
            }
        }
    }

    let mut renames = BTreeMap::<PathBuf, PathBuf>::new();

    // Write and Delete
    for existing_file_info in &existing_files {
        let old_path = &existing_file_info.path;
        let hash = &existing_file_info.hash;

        if let Some(file_info) = updated_files.get(&existing_file_info.idx) {
            let new_path = &file_info.new_path;
            if old_path != new_path {
                //println!("Rename {} to {}", old_path.display(), new_path.display());
                renames.insert(old_path.clone(), new_path.clone());
            }

            if let Some(content) = file_info.content {
                if hash != &compute_hash(&mut content.as_bytes()).unwrap() {
                    println!("Write {:?} to {}", content, old_path.display());
                }
            }
        } else {
            println!("Delete {}", old_path.display());
        }
    }

    loop {
        let v = match renames.keys().next() {
            Some(v) => v,
            None => break,
        };

        process_rename(
            v.clone(),
            &mut Context {
                renames: &mut renames,
                processed: &mut HashSet::new(),
            },
        );
    }

    Ok(())
}

struct Context<'t> {
    renames: &'t mut BTreeMap<PathBuf, PathBuf>,
    processed: &'t mut HashSet<PathBuf>,
}

use std::str::FromStr;

fn process_rename<'t>(from: PathBuf, context: &mut Context<'t>) {
    match context.renames.get(&from) {
        None => {}
        Some(to) => {
            context.processed.insert(from.clone());
            if context.processed.contains(to) {
                let tmp = PathBuf::from_str(&"temp").unwrap();
                println!("Rename from {:?} to {:?}", from, tmp);
                context.renames.insert(tmp, to.clone());
            } else {
                let to = to.clone();
                process_rename(to.clone(), context);
                println!("Rename from {:?} to {:?}", from, to);
            }
            context.renames.remove(&from);
        }
    };
}
