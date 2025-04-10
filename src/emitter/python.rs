use std::{collections::HashSet, io::Write};

use crate::parser::{AtomicType, Struct, Type};

use super::Emitter;

#[derive(Debug, PartialEq, Clone)]
pub struct PyEmitter {
    seen: HashSet<String>,
}

type R = std::io::Result<()>;

impl PyEmitter {
    pub fn new() -> Self {
        Self {
            seen: Default::default(),
        }
    }
    fn emit_header(&self, f: &mut dyn Write) -> R {
        write!(f, "from pydantic import BaseModel as Schema\n")?;
        write!(f, "\n\n")
    }
    fn get_type_name(&self, typ: &Type) -> String {
        match typ {
            Type::User(name) => {
                if self.seen.contains(name) {
                    name.clone()
                } else {
                    format!(r"'{name}'")
                }
            }
            Type::List(inner) => {
                format!("list[{}]", self.get_type_name(inner))
            }
            Type::Builtin(tp) => match tp {
                AtomicType::U8
                | AtomicType::U16
                | AtomicType::U32
                | AtomicType::U64
                | AtomicType::USize
                | AtomicType::UInt
                | AtomicType::I8
                | AtomicType::I16
                | AtomicType::I32
                | AtomicType::I64
                | AtomicType::ISize
                | AtomicType::Int => "int".to_string(),
                AtomicType::F32 | AtomicType::F64 => "float".to_string(),
                AtomicType::String => "str".to_string(),
                AtomicType::Binary => "bytes".to_string(),
            },
        }
    }
    fn emit_struct(&mut self, f: &mut dyn Write, st: &Struct) -> R {
        write!(f, "class {}(Schema):\n", st.name)?;
        if st.fields.is_empty() {
            write!(f, "    pass\n")?;
        } else {
            for field in &st.fields {
                write!(
                    f,
                    "    {}: {}\n",
                    field.name,
                    self.get_type_name(&field.type_)
                )?;
            }
        }
        write!(f, "\n\n")?;
        self.seen.insert(st.name.clone());
        Ok(())
    }
}

impl Emitter for PyEmitter {
    fn emit(&mut self, document: &crate::parser::Document, writer: &mut dyn std::io::Write) -> R {
        self.emit_header(writer)?;
        for st in &document.user_types {
            self.emit_struct(writer, st)?;
        }
        Ok(())
    }
}
