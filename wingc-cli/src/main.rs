use std::{io::BufWriter, path::PathBuf};

use wingc::{emitter, parser, semantic_analyzer};

use argh::FromArgs;
use emitter::{Emitter, PyEmitter, RustEmitter};
use parser::parse_document;
use semantic_analyzer::analyze_errors;

use miette::{Context, Result as R};
use miette::{IntoDiagnostic, bail};

#[derive(FromArgs)]
#[argh(
    description = "Given a wing file, generates a the corresponding definitions in the target programming language"
)]
pub struct Args {
    #[argh(positional)]
    input: PathBuf,
    #[argh(option, short = 'l')]
    #[argh(
        description = "specifies the language to be emitted. If not specified, will try to look into the file extension."
    )]
    language: Option<String>,
    #[argh(positional)]
    output: Option<PathBuf>,
}

impl Args {
    fn get_emitter(&self) -> R<Box<dyn Emitter>> {
        let language = match self.language {
            Some(ref o) => o,
            None => self
                .output
                .as_ref()
                .context("Expected output file to be specified when no language is set")?
                .extension()
                .context("Expected output file to have an extension")?
                .to_str()
                .context("Can't parse extension")?,
        };
        Ok(match language {
            "py" | "python" => Box::new(PyEmitter::new()),
            "rs" | "rust" => Box::new(RustEmitter::new()),
            _ => {
                bail!("No emitter available for '{language}'.");
            }
        })
    }
}

fn main() -> R<()> {
    let args: Args = argh::from_env();
    let mut emitter: Box<dyn Emitter> = args.get_emitter()?;
    let input = &args.input;
    if input.extension().map(|ext| ext != "wing").unwrap_or(true) {
        bail!("Input file is not a .wing file");
    }
    let input_data = std::fs::read_to_string(input).expect("Failed to read input file");
    let document = parse_document(&*input_data).expect("Failed to parse document");
    if let Err(err) = analyze_errors(&document) {
        bail!("{:?}", err.with_source_code(input_data));
    }
    let mut output: &mut dyn std::io::Write = if let Some(output) = args.output {
        &mut std::fs::File::create(output)
            .map(BufWriter::new)
            .into_diagnostic()
            .context("Failed to create output file")?
    } else {
        &mut std::io::stdout()
    };
    emitter.emit(&document, &mut output).unwrap();
    Ok(())
}
