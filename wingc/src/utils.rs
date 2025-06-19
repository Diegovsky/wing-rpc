use std::path::Path;

use crate::{
    emitter::{Emitter, PyEmitter, RustEmitter},
    parser, semantic_analyzer,
};

#[derive(Clone, Copy)]
pub enum Lang {
    Rust,
    Python,
}

#[derive(Clone)]
pub enum Mode {
    Test,
    Emit,
}

const LANGS: &[Lang] = &[Lang::Rust, Lang::Python];

pub fn test(name: &str, mode: Mode) {
    let cwd = Path::new("test-files/");
    let input = std::fs::read_to_string(cwd.join(format!("{name}.wing"))).unwrap();
    let doc = parser::parse_document(&input).expect("Failed to parse document");
    semantic_analyzer::analyze_errors(&doc).unwrap();
    let mut output = Vec::new();
    for lang in LANGS {
        let (mut emitter, ext): (Box<dyn Emitter>, &str) = match lang {
            Lang::Rust => (Box::new(RustEmitter::new()), "rs"),
            Lang::Python => (Box::new(PyEmitter::new()), "py"),
        };
        let output_file = cwd.join(format!("{name}.{ext}"));
        output.clear();
        emitter.emit(&doc, &mut output).unwrap();
        let output = std::str::from_utf8(output.as_slice()).expect("Got invalid utf-8 sequence");
        match mode {
            Mode::Test => {
                let expected = std::fs::read_to_string(&output_file).unwrap();
                assert_eq!(expected, output)
            }
            Mode::Emit => {
                std::fs::write(&output_file, output).unwrap();
            }
        }
    }
}
