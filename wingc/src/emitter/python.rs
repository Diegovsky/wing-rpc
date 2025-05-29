use std::{collections::HashSet, io::Write};

use crate::parser::{Builtin, StructField, Type, UserType};

use super::Emitter;

#[derive(Debug, PartialEq, Clone)]
pub struct PyEmitter {
    seen: HashSet<String>,
    indent: usize,
}

type R = std::io::Result<()>;

impl PyEmitter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            seen: Default::default(),
        }
    }
    fn emit_header(&self, f: &mut dyn Write) -> R {
        write!(f, "from wing_rpc import Schema, Enum\n")?;
        write!(f, "from typing import ClassVar\n")?;
        write!(f, "from enum import StrEnum\n")?;
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
                Builtin::U8
                | Builtin::U16
                | Builtin::U32
                | Builtin::U64
                | Builtin::USize
                | Builtin::UInt
                | Builtin::I8
                | Builtin::I16
                | Builtin::I32
                | Builtin::I64
                | Builtin::ISize
                | Builtin::Int => "int",
                Builtin::F32 | Builtin::F64 => "float",
                Builtin::Bool => "bool",
                Builtin::String => "str",
                Builtin::Binary => "bytes",
            }
            .to_string(),
        }
    }
    fn ident(&self, f: &mut dyn Write) -> R {
        write!(f, "{}", " ".repeat(self.indent * 4))
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
        write!(f, "class {}(StrEnum):\n", name)?;
        self.indent += 1;
        for (name, value) in variants {
            self.emit_field(
                f,
                name.as_ref(),
                None,
                format!("'{}'", value.as_ref()).as_str(),
            )?;
        }
        self.indent -= 1;
        Ok(())
    }
    fn get_base_class(&self, utype: &UserType) -> &str {
        match utype {
            UserType::Struct(_) => "Schema",
            UserType::Enum(_) => "Enum",
        }
    }
    fn emit_match_args<'a>(&self, fields: impl Iterator<Item = &'a str>, f: &mut dyn Write) -> R {
        let match_args = format!(
            "({},)",
            fields
                .map(|field| format!("'{}'", field))
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.emit_field(f, "__match_args__", "ClassVar[tuple]", match_args.as_ref())?;
        Ok(())
    }
    fn emit_user_type(&mut self, f: &mut dyn Write, utype: &UserType) -> R {
        // Emit inner children types
        for child in utype.children_user_types() {
            self.emit_user_type(f, child)?;
        }
        self.ident(f)?;
        write!(
            f,
            "class {}({}):\n",
            utype.name(),
            self.get_base_class(utype)
        )?;
        self.indent += 1;
        if utype.is_empty() {
            self.ident(f)?;
            write!(f, "pass\n")?;
        } else {
            match utype {
                UserType::Struct(st) => {
                    self.emit_match_args(st.fields.iter().map(|f| f.name.as_str()), f)?;
                    for field in &st.fields {
                        let StructField { name, typ: type_ } = &field.value;
                        self.emit_field(f, name.as_str(), self.get_type_name(type_).as_str(), None)?
                    }
                }
                UserType::Enum(en) => {
                    self.emit_match_args(["tag", "value"].into_iter(), f)?;
                    let varnames: Vec<String> =
                        en.variants().map(|t| self.get_type_name(&t.typ)).collect();
                    self.emit_python_strenum(
                        f,
                        "Tag",
                        en.variants().map(|t| t.value.name).map(|t| (t.clone(), t)),
                    )?;
                    self.emit_field(f, "tag", "Tag", None)?;
                    self.emit_field(f, "value", varnames.join(" | ").as_str(), None)?;
                }
            }
        }
        self.indent -= 1;
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
