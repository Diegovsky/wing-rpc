use std::io::Write;

use crate::parser::Document;

mod python;
mod rust;

pub use python::PyEmitter;
pub use rust::RustEmitter;

pub trait Emitter {
    fn emit(&mut self, document: &Document, writer: &mut dyn Write) -> std::io::Result<()>;
}
