use super::{ExistingFileInfo, ParsedMerchDoc, UpdatedFileInfo};
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::{path::PathBuf, str::FromStr};

pub fn parse_merch_doc<'t>(content: &'t str) -> ParsedMerchDoc<'t> {
    let merch_instruction_regex =
        RegexBuilder::new(r"^.*? merch::(?P<instruction>.*?)(?P<colon>:)?\r?\n")
            .multi_line(true)
            .build()
            .unwrap();

    let instruction_regex =
        Regex::new(&"(?P<command>[a-zA-Z0-9-]+):\\s*(?P<args>(<.*?>\\s*)*)").unwrap();
    let arg_regex = Regex::new(&"<(?P<arg>.*?)>").unwrap();

    let mut existing_files: Vec<ExistingFileInfo> = Vec::new();
    let mut updated_files: HashMap<usize, UpdatedFileInfo> = HashMap::new();
    let mut base_path: Option<PathBuf> = None;

    struct ExpectContentInfo {
        content_end: usize,
        file_idx: usize,
    }

    let mut last_match: Option<ExpectContentInfo> = None;

    //let mut last_file_idx = None;
    //let mut last_full_match: Option<regex::Match> = None;
    for instruction_match in merch_instruction_regex
        .captures_iter(&content)
        .map(|m| Some(m))
        .chain(vec![None])
    {
        // Set new content
        if let Some(ExpectContentInfo {
            content_end,
            file_idx,
        }) = last_match
        {
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

            let text_in_between = &content[content_end..end];
            let file_info = updated_files.get_mut(&file_idx).unwrap();
            file_info.content = Some(text_in_between);
        }

        if let Some(ref instruction_match) = instruction_match {
            let has_colon = instruction_match.name(&"colon").is_some();
            let full_match = instruction_match.get(0).unwrap();

            let instruction = instruction_match.name(&"instruction").unwrap().as_str();
            let m = instruction_regex.captures(&instruction).unwrap();
            let command = m.name(&"command").unwrap().as_str();
            let args = m.name(&"args").unwrap().as_str();
            let args: Vec<_> = arg_regex
                .captures_iter(&args)
                .map(|c| c.name(&"arg").unwrap().as_str())
                .collect();

            last_match = None;

            match command {
                "setup" => {
                    base_path = Some(PathBuf::from_str(args[0]).unwrap());
                }
                "existing-file" => {
                    let path: PathBuf = args[0].parse().unwrap();
                    let idx: usize = args[1].parse().unwrap();
                    let hash = args[2].to_owned();

                    existing_files.push(ExistingFileInfo { idx, path, hash });
                }
                "file" => {
                    let new_path: PathBuf = args[0].parse().unwrap();
                    let idx: usize = args[1].parse().unwrap();

                    updated_files.insert(
                        idx,
                        UpdatedFileInfo {
                            idx,
                            new_path,
                            content: None,
                        },
                    );

                    if has_colon {
                        last_match = Some(ExpectContentInfo {
                            content_end: full_match.end(),
                            file_idx: idx,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    ParsedMerchDoc {
        existing_files,
        updated_files,
        base_path: base_path.unwrap(),
    }
}
