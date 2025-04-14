use std::{borrow::Cow, collections::HashSet, io::Write};

use crate::parser::{AtomicType, Enum, Struct, StructField, Type, UserType};

use super::Emitter;

#[derive(Debug, PartialEq, Clone)]
pub struct PyEmitter {
    seen: HashSet<String>,
    ident: usize,
}

type R = std::io::Result<()>;

impl PyEmitter {
    pub fn new() -> Self {
        Self {
            ident: 0,
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
    fn ident(&self, f: &mut dyn Write) -> R {
        write!(f, "{}", " ".repeat(self.ident * 4))
    }
    fn emit_field<'a>(
        &self,
        f: &mut dyn Write,
        name: &str,
        typ: impl Into<Option<&'a str>>,
        value: impl Into<Option<&'a str>>,
    ) -> R {
        self.ident(f)?;
        write!(f, "{name}")?;
        if let Some(typ) = typ.into() {
            write!(f, ": {}", typ)?;
        }
        if let Some(value) = value.into() {
            write!(f, " = {}", value)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
    fn emit_python_strenum<'a>(
        &mut self,
        f: &mut dyn Write,
        name: &str,
        variants: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    ) -> R {
        self.ident(f)?;
        write!(f, "class {}(Schema):\n", name)?;
        self.ident += 1;
        for (name, value) in variants {
            self.emit_field(f, name.as_ref(), None, value.as_ref())?;
        }
        self.ident -= 1;
        Ok(())
    }
    fn emit_user_type(&mut self, f: &mut dyn Write, utype: &UserType) -> R {
        self.ident(f)?;
        write!(f, "class {}(Schema):\n", utype.name())?;
        if utype.is_empty() {
            self.ident(f)?;
            write!(f, "pass\n")?;
        } else {
            self.ident += 1;
            match utype {
                UserType::Struct(st) => {
                    for StructField { name, type_ } in &st.fields {
                        self.emit_field(f, name.as_str(), self.get_type_name(type_).as_str(), None)?
                    }
                }
                UserType::Enum(en) => {
                    let varnames: Vec<String> =
                        en.variants.iter().map(|t| self.get_type_name(t)).collect();
                    self.emit_python_strenum(
                        f,
                        "Tag",
                        varnames.iter().map(|vn| (vn.replace("'", ""), vn)),
                    )?;
                    self.emit_field(f, "tag", "Tag", None)?;
                    self.emit_field(f, "value", varnames.join(" | ").as_str(), None)?;
                }
            }
            self.ident -= 1;
        }
        write!(f, "\n\n")?;
        self.seen.insert(utype.name().to_owned());
        Ok(())
    }
}

impl Emitter for PyEmitter {
    fn emit(&mut self, document: &crate::parser::Document, writer: &mut dyn std::io::Write) -> R {
        self.emit_header(writer)?;
        for utype in document.user_types.iter() {
            self.emit_user_type(writer, utype)?;
        }
        Ok(())
    }
}
