use std::fmt;
use std::io::Error;
use std::path::PathBuf;

pub enum Action<'content> {
    Write(PathBuf, &'content str),
    Rename(PathBuf, PathBuf),
    Delete(PathBuf),
}

impl Action<'_> {
    pub fn run(&self) -> Result<(), Error> {
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
            Action::Write(path, _content) => write!(f, "Update \"{}\"", path.display()),
            Action::Rename(old, new) => {
                write!(f, "Rename \"{}\" to \"{}\"", old.display(), new.display())
            }
            Action::Delete(path) => write!(f, "Delete \"{}\"", path.display()),
        }
    }
}
