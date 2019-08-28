use blake2::digest::VariableOutput;
use blake2::VarBlake2b;
use std::io::{copy, Error, Read, Write};

pub fn compute_hash(read: &mut impl Read) -> Result<String, Error> {
    let mut hasher = VarBlake2b::new(10).unwrap();
    copy(read, &mut hasher)?;
    let hash = hex::encode(hasher.vec_result());
    Ok(hash)
}

pub struct LineFormatter {
    pub prefix: String,
    pub suffix: String,
}

impl LineFormatter {
    pub fn writeln<W: Write>(&self, writer: &mut W, text: String) -> Result<(), std::io::Error> {
        writeln!(writer, "{}{}{}", self.prefix, text, self.suffix)
    }
}
