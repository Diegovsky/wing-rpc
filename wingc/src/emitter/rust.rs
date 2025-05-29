use std::{collections::HashMap, io::Write};

use crate::parser::{Builtin, EnumVariant, Type, UserType};

use super::Emitter;

#[derive(Debug, PartialEq, Clone)]
pub struct RustEmitter {
    indent: usize,
    user_types: HashMap<String, UserType>,
}

impl RustEmitter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            user_types: Default::default(),
        }
    }
    fn indent(&self, f: &mut dyn Write) -> R {
        write!(f, "{}", " ".repeat(self.indent * 4))
    }
    fn emit_header(&self, f: &mut dyn Write) -> R {
        write!(f, "use serde::{{Serialize, Deserialize}};\n")?;
        write!(f, "use wing_rpc::Message as WingMessage;\n")?;
        write!(f, "\n\n")
    }
    fn is_ut_partialeq(&self, ut: &UserType) -> bool {
        ut.children_types().all(|tp| self.is_partialeq(&tp.value))
    }
    fn is_partialeq(&self, typ: &Type) -> bool {
        match typ {
            Type::Builtin(Builtin::F32 | Builtin::F64) => false,
            Type::Builtin(_) => true,
            Type::List(tp) => self.is_partialeq(tp),
            Type::User(ut) => self.is_ut_partialeq(&self.user_types[dbg!(ut)]),
        }
    }
    fn get_type_name(&self, typ: &Type) -> String {
        match typ {
            Type::User(name) => name.to_string(),
            Type::List(inner) => {
                format!("Vec<{}>", self.get_type_name(inner))
            }
            Type::Builtin(tp) => match tp {
                Builtin::USize
                | Builtin::ISize
                | Builtin::U8
                | Builtin::U16
                | Builtin::U32
                | Builtin::U64
                | Builtin::I8
                | Builtin::I16
                | Builtin::I32
                | Builtin::I64
                | Builtin::F32
                | Builtin::Bool
                | Builtin::F64 => <&str>::from(tp),
                Builtin::UInt => "u32",
                Builtin::Int => "i32",
                Builtin::String => "String",
                Builtin::Binary => "Vec<u8>",
            }
            .to_string(),
        }
    }
    fn emit_user_type(&mut self, f: &mut dyn Write, ut: &UserType) -> R {
        // Emit inner children types
        for child in ut.children_user_types() {
            self.emit_user_type(f, child)?;
        }
        let mut derives = vec!["Debug", "Clone"];
        if self.is_ut_partialeq(ut) {
            derives.push("PartialEq");
        }
        derives.extend(["Serialize", "Deserialize"]);
        self.indent(f)?;
        write!(f, "#[derive({})]\n", derives.join(", "))?;
        let name = ut.name();
        self.indent(f)?;
        write!(f, "pub ")?;
        match ut {
            UserType::Struct(st) => {
                write!(f, "struct {} {{\n", name)?;
                self.indent += 1;
                for field in st.fields.iter() {
                    self.indent(f)?;
                    write!(
                        f,
                        "pub {}: {},\n",
                        field.name,
                        self.get_type_name(&field.typ)
                    )?;
                }
                self.indent -= 1;
                f.write_all(b"}\n\n")?;
            }
            UserType::Enum(en) => {
                write!(f, "enum {} {{\n", name)?;
                self.indent += 1;
                for field in en.definitions.iter() {
                    match &field.value {
                        EnumVariant::NamedVariant(field) => {
                            self.indent(f)?;
                            let name = &*field.name;
                            let tp = self.get_type_name(&field.typ);
                            write!(f, "{}({}),\n", name, tp)?;
                        }
                        _ => (),
                    }
                }
                self.indent -= 1;
                f.write_all(b"}\n\n")?;
            }
        }

        write!(f, "impl<'a> WingMessage<'a> for {name} {{\n")?;
        self.indent += 1;
        self.indent(f)?;
        write!(f, "const NAME: &'static str = \"{name}\";\n")?;
        self.indent -= 1;
        write!(f, "}}\n\n")?;

        Ok(())
    }

    fn register_ut(&mut self, ut: &UserType) {
        self.user_types.insert(ut.name().into(), ut.clone());
        for child in ut.children_user_types() {
            self.register_ut(child);
        }
    }
}

type R = std::io::Result<()>;

impl Emitter for RustEmitter {
    fn emit(&mut self, document: &crate::parser::Document, writer: &mut dyn std::io::Write) -> R {
        self.emit_header(writer)?;
        self.user_types.clear();

        for ut in &document.user_types {
            self.register_ut(&ut.value);
        }
        for (k, v) in &self.user_types {
            println!("{k}: {}", v.name());
        }
        for ut in document.user_types.iter() {
            self.emit_user_type(writer, ut)?;
        }
        Ok(())
    }
}
