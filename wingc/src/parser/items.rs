use std::ops::Deref;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Builtin {
    // Specific Int Sizes
    U8,
    U16,
    U32,
    U64,
    USize,
    I8,
    I16,
    I32,
    I64,
    ISize,

    // General types
    UInt,
    Int,
    F32,
    F64,
    Bool,
    String,
    Binary,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Builtin(Builtin),
    List(Box<Type>),
    User(String),
    UserInline(UserType),
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum EnumVariant {
    NamedVariant(StructField),
    UserType(UserType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub definitions: SVec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: SVec<StructField>,
}

#[derive(Debug, Clone, From, PartialEq)]
pub enum UserType {
    Struct(S<Struct>),
    Enum(S<Enum>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub user_types: SVec<UserType>,
}
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin(bt) => write!(f, "{}", <&str>::from(bt)),
            Self::List(inner) => write!(f, "List<{}>", inner),
            Self::User(name) => write!(f, "{}", name),
            Self::UserInline(user) => write!(f, "{}", user.name()),
        }
    }
}
use easy_ext::ext;
#[ext]
impl<T> Vec<T> {
    fn with(mut self, val: T) -> Self {
        self.push(val);
        self
    }
}

impl Struct {
    pub fn children_user_types<'a>(&'a self) -> Vec<&'a UserType> {
        let val = self
            .fields
            .iter()
            .flat_map(|def| match &def.typ {
                Type::UserInline(user) => user.children_user_types(),
                _ => vec![],
            })
            .collect();
        val
    }
}

impl UserType {
    pub fn children_user_types<'a>(&'a self) -> Vec<&'a UserType> {
        match self {
            UserType::Enum(en) => en.children_user_types(),
            UserType::Struct(st) => st.children_user_types(),
        }
        .with(self)
    }
}

impl Enum {
    pub fn children_user_types<'a>(&'a self) -> Vec<&'a UserType> {
        self.definitions
            .iter()
            .flat_map(|def| match &def.value {
                EnumVariant::UserType(ut) => ut.children_user_types(),
                EnumVariant::NamedVariant(StructField {
                    typ: Type::UserInline(user),
                    ..
                }) => user.children_user_types(),
                _ => vec![],
            })
            .collect()
    }
    pub fn variants(&self) -> impl Iterator<Item = S<StructField>> {
        self.definitions.iter().map(|var| {
            var.as_ref().map(|var| match var {
                EnumVariant::NamedVariant(f) => f.clone(),
                EnumVariant::UserType(ut) => StructField {
                    name: ut.name().into(),
                    typ: Type::User(ut.name().into()),
                },
            })
        })
    }
}

impl Type {
    pub fn as_user(&self) -> Option<&str> {
        if let Type::User(tp) = self {
            Some(tp)
        } else {
            None
        }
    }
}
impl UserType {
    pub fn name(&self) -> &str {
        match self {
            Self::Struct(st) => &st.name,
            Self::Enum(en) => &en.name,
        }
    }
    pub fn children_types<'a>(&'a self) -> impl Iterator<Item = S<Type>> {
        let iter: Box<dyn Iterator<Item = S<StructField>>> = match self {
            Self::Struct(st) => Box::new(st.fields.iter().cloned()),
            Self::Enum(en) => Box::new(en.variants()),
        };
        iter.map(|fd| fd.as_ref().map(|fd| fd.typ.clone()))
    }
    pub fn is_empty(&self) -> bool {
        match self {
            UserType::Struct(st) => st.fields.is_empty(),
            UserType::Enum(en) => en.definitions.is_empty(),
        }
    }
}
