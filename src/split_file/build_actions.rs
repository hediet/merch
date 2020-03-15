use super::{Action, ParsedMerchDoc};
use crate::compute_hash;
use std::{
    collections::{BTreeMap, HashSet},
    ffi::OsString,
    path::PathBuf,
};

pub fn build_actions(doc: ParsedMerchDoc) -> Vec<Action> {
    let mut actions = Vec::<Action>::new();
    let mut renames = BTreeMap::<PathBuf, PathBuf>::new();

    let ParsedMerchDoc {
        base_path,
        existing_files,
        updated_files,
    } = doc;

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
                    // write to the old path before renaming it.
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
                pending_renames: &mut renames,
                processed: &mut HashSet::new(),
                actions: &mut actions,
            },
        );
    }

    actions
}

struct Context<'t, 'actions, 'content> {
    // old name -> new name
    pending_renames: &'t mut BTreeMap<PathBuf, PathBuf>,
    // processed old names
    processed: &'t mut HashSet<PathBuf>,
    actions: &'actions mut Vec<Action<'content>>,
}

fn process_rename<'t, 'actions, 'content>(
    from: PathBuf,
    context: &mut Context<'t, 'actions, 'content>,
) {
    match context.pending_renames.get(&from) {
        None => {}
        Some(to) => {
            let is_new = context.processed.insert(from.clone());
            if !is_new {
                panic!("Something is fishy, `from` should not have been processed yet.")
            }

            if context.processed.contains(to) {
                let mut tmp = to.clone();

                let mut file_name = OsString::new();
                file_name.push(tmp.file_name().unwrap());
                file_name.push("_temp");

                tmp.set_file_name(file_name);

                let to_cloned = to.clone();
                context.pending_renames.insert(tmp.clone(), to_cloned);
                context.actions.push(Action::Rename(from.clone(), tmp));
            } else {
                let to = to.clone();
                process_rename(to.clone(), context);
                context.actions.push(Action::Rename(from.clone(), to));
            }
            context.pending_renames.remove(&from);
        }
    };
}
