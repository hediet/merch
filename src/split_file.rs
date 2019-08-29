use super::compute_hash;
use regex::{Regex, RegexBuilder};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
use std::fmt;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;

enum Action<'content> {
    Write(PathBuf, &'content str),
    Rename(PathBuf, PathBuf),
    Delete(PathBuf),
}

impl Action<'_> {
    fn perform(&self) -> Result<(), Error> {
        match self {
            Action::Write(path, content) => std::fs::write(path, content),
            Action::Rename(old, new) => std::fs::rename(old, new),
            Action::Delete(path) => std::fs::remove_file(path),
        }
    }
}

impl<'t> fmt::Display for Action<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Write(path, content) => write!(f, "Write \"{}\": {}", path.display(), content),
            Action::Rename(old, new) => {
                write!(f, "Rename \"{}\" to \"{}\"", old.display(), new.display())
            }
            Action::Delete(path) => write!(f, "Delete \"{}\"", path.display()),
        }
    }
}

pub fn split_file<R: Read>(input: &mut R) -> Result<(), Error> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;

    let mut renames = BTreeMap::<PathBuf, PathBuf>::new();

    let Doc {
        existing_files,
        updated_files,
        base_path,
    } = parse(&content);

    let mut actions = Vec::<Action>::new();

    // Write and Delete
    for existing_file_info in &existing_files {
        let old_path = base_path.join(&existing_file_info.path);

        let hash = &existing_file_info.hash;

        if let Some(file_info) = updated_files.get(&existing_file_info.idx) {
            let new_path = base_path.join(&file_info.new_path);
            if old_path != new_path {
                renames.insert(old_path.clone(), new_path.clone());
            }

            if let Some(content) = file_info.content {
                if hash != &compute_hash(&mut content.as_bytes()).unwrap() {
                    actions.push(Action::Write(old_path.clone(), content));
                }
            }
        } else {
            actions.push(Action::Delete(old_path.clone()));
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
                actions: &mut actions,
            },
        );
    }

    for action in actions {
        println!("{}", action);
        action.perform().unwrap();
    }

    Ok(())
}

#[derive(Debug)]
struct Doc<'t> {
    existing_files: Vec<ExistingFileInfo>,
    updated_files: HashMap<usize, UpdatedFileInfo<'t>>,
    base_path: PathBuf,
}

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

use std::str::FromStr;

fn parse<'t>(content: &'t str) -> Doc<'t> {
    let merch_instruction_regex = RegexBuilder::new(r"^.*? merch::(?P<instruction>.*).*?\r?\n")
        .multi_line(true)
        .build()
        .unwrap();

    let instruction_regex =
        Regex::new(&"(?P<command>[a-zA-Z0-9-]+):\\s*(?P<args>(<.*?>\\s*)*)").unwrap();
    let arg_regex = Regex::new(&"<(?P<arg>.*?)>").unwrap();

    let mut existing_files: Vec<ExistingFileInfo> = Vec::new();
    let mut updated_files: HashMap<usize, UpdatedFileInfo> = HashMap::new();
    let mut base_path: Option<PathBuf> = None;

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
                "setup" => {
                    base_path = Some(PathBuf::from_str(args[0]).unwrap());
                }
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

    Doc {
        existing_files,
        updated_files,
        base_path: base_path.unwrap(),
    }
}

struct Context<'t, 'actions, 'content> {
    renames: &'t mut BTreeMap<PathBuf, PathBuf>,
    processed: &'t mut HashSet<PathBuf>,
    actions: &'actions mut Vec<Action<'content>>,
}

fn process_rename<'t, 'actions, 'content>(
    from: PathBuf,
    context: &mut Context<'t, 'actions, 'content>,
) {
    match context.renames.get(&from) {
        None => {}
        Some(to) => {
            context.processed.insert(from.clone());
            if context.processed.contains(to) {
                let mut tmp = to.clone();
                let mut file_name = OsString::new();
                file_name.push(tmp.file_name().unwrap());
                file_name.push("_temp");
                tmp.set_file_name(file_name);
                context.renames.insert(tmp.clone(), to.clone());
                context.actions.push(Action::Rename(from.clone(), tmp));
            } else {
                let to = to.clone();
                process_rename(to.clone(), context);
                context.actions.push(Action::Rename(from.clone(), to));
            }
            context.renames.remove(&from);
        }
    };
}
