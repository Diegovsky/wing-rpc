use super::Emitter;

#[derive(Debug, PartialEq, Clone)]
pub struct RustEmitter {}

impl RustEmitter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Emitter for RustEmitter {
    fn emit(
        &mut self,
        document: &crate::parser::Document,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        todo!()
    }
}
