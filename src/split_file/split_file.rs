use super::{build_actions, parse_merch_doc};
use std::io::{Error, Read};

pub fn split_file<R: Read>(input: &mut R, dry_run: bool) -> Result<(), Error> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;

    let doc = parse_merch_doc(&content);
    let actions = build_actions(doc);

    for action in actions {
        println!("{}", action);
        if !dry_run {
            action.run().unwrap();
        }
    }

    Ok(())
}
