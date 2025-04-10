use std::{collections::VecDeque, ffi::OsString, io::BufWriter, path::PathBuf};

use emitter::{Emitter, PyEmitter, RustEmitter};
use parser::parse_document;

mod emitter;
mod parser;

fn main() {
    let mut args: VecDeque<OsString> = std::env::args_os().skip(1).collect();
    let input = PathBuf::from(args.pop_front().expect("input file"));
    let output = PathBuf::from(args.pop_front().expect("Output file"));
    let mut emitter: Box<dyn Emitter> = match output
        .extension()
        .expect("Expected output file to have an extension")
        .to_str()
        .expect("Can't parse extension")
    {
        "py" => Box::new(PyEmitter::new()),
        "rs" => Box::new(RustEmitter::new()),
        _ => panic!("No emitter available for the file {output:?}"),
    };
    if input.extension().map(|ext| ext != "wing").unwrap_or(true) {
        panic!("Input file is not a .wing file");
    }
    let input_data = std::fs::read_to_string(input).expect("Failed to read input file");
    let document = parse_document(&*input_data).expect("Failed to parse document");
    let mut output: &mut dyn std::io::Write = if output.file_stem().unwrap() != "-" {
        &mut std::fs::File::create(output)
            .map(BufWriter::new)
            .expect("Failed to create output file")
    } else {
        &mut std::io::stdout()
    };
    emitter.emit(&document, &mut output).unwrap();
}
