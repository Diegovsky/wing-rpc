use std::str::FromStr;

use derive_more::From;
use pest::{
    Parser, RuleType,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use serde::Deserialize;
use strum::{EnumString, IntoStaticStr};

#[derive(Parser, Debug, Clone, PartialEq)]
#[grammar = "./idl.pest"]
pub(crate) struct WingParser;

mod rules;
pub mod span;
use rules::ParseItem;
pub use span::{S, SVec};

#[easy_ext::ext]
pub(crate) impl<R: RuleType + Send + Sync + 'static, P: Parser<R>> P {
    fn parse2(rule: R, input: &str) -> miette::Result<Pairs<'_, R>> {
        Self::parse(rule, input)
            .map_err(pest::error::Error::into_miette)
            .map_err(miette::Report::from)
    }
}

#[easy_ext::ext]
pub(crate) impl<'i> Pairs<'i, Rule> {
    fn next_item<P: ParseItem>(&mut self) -> miette::Result<P> {
        P::parse(self.next2())
    }
    fn collect_items<P: ParseItem>(&mut self) -> miette::Result<Vec<P>> {
        self.map(|pair| P::parse(pair)).collect()
    }
    #[track_caller]
    fn next2(&mut self) -> Pair<'i, Rule> {
        self.next().unwrap()
    }
}
pub fn parse_document(text: &str) -> miette::Result<Document> {
    let mut tokens = WingParser::parse2(Rule::document, text)?;
    let doc: S<Document> = tokens.next_item()?;
    let remaining = text[doc.span.end()..].trim();
    // Well, if there is remaining doc text, parsing failed.
    if remaining.len() > 0 {
        return parse_document(remaining);
    }
    Ok(doc.value)
}

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

#[derive(Deserialize)]
enum TypeDe {
    Ident(String),
    #[serde(rename = "list_type")]
    List(Box<TypeDe>),
}

impl From<TypeDe> for Type {
    fn from(value: TypeDe) -> Self {
        match value {
            TypeDe::List(lst) => Self::List(Box::new((*lst).into())),
            TypeDe::Ident(ident) => match Builtin::from_str(&*ident).ok() {
                Some(builtin) => Self::Builtin(builtin),
                None => Self::User(ident),
            },
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
#[serde(from = "TypeDe")]
pub enum Type {
    Builtin(Builtin),
    List(Box<Type>),
    User(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin(bt) => write!(f, "{}", <&str>::from(bt)),
            Self::List(inner) => write!(f, "List<{}>", inner),
            Self::User(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, From, Deserialize)]
pub enum EnumVariant {
    NamedVariant(StructField),
    UserType(UserType),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Enum {
    pub name: String,
    pub definitions: SVec<EnumVariant>,
}

impl Struct {
    pub fn children_user_types<'a>(&'a self) -> impl Iterator<Item = &'a UserType> {
        std::iter::empty()
    }
}

impl UserType {
    pub fn children_user_types<'a>(&'a self) -> Vec<&'a UserType> {
        match self {
            UserType::Enum(en) => en.children_user_types().collect(),
            UserType::Struct(st) => st.children_user_types().collect(),
        }
    }
}

impl Enum {
    pub fn children_user_types<'a>(&'a self) -> impl Iterator<Item = &'a UserType> {
        self.definitions.iter().filter_map(|def| match &def.value {
            EnumVariant::UserType(ut) => Some(ut),
            EnumVariant::NamedVariant(_) => None,
        })
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StructField {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Struct {
    pub name: String,
    pub fields: SVec<StructField>,
}

#[derive(Debug, Clone, From, PartialEq, Deserialize)]
pub enum UserType {
    Struct(S<Struct>),
    Enum(S<Enum>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Document {
    pub user_types: SVec<UserType>,
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

#[cfg(test)]
pub mod test {
    use crate::pest_deserializer::PestDeserializer;

    use super::span::Spanned;
    use super::*;

    impl From<Builtin> for Type {
        fn from(val: Builtin) -> Self {
            Type::Builtin(val)
        }
    }

    impl From<Struct> for UserType {
        fn from(value: Struct) -> Self {
            UserType::Struct(S::new_unspanned(value))
        }
    }

    impl From<Enum> for UserType {
        fn from(value: Enum) -> Self {
            UserType::Enum(S::new_unspanned(value))
        }
    }

    impl From<&str> for Type {
        fn from(val: &str) -> Self {
            Type::User(s(val))
        }
    }
    impl StructField {
        pub fn new(name: impl Into<String>, type_: impl Into<Type>) -> Self {
            Self {
                name: name.into(),
                typ: type_.into(),
            }
        }
    }

    impl Type {
        fn list(inner: impl Into<Type>) -> Self {
            Self::List(Box::new(inner.into()))
        }
    }

    fn s(text: &'_ str) -> String {
        text.to_owned()
    }

    macro_rules! assert_parse {
        ($left:expr, $right:expr) => {
            assert_parse!($left, $right, Document)
        };

        ($left:expr, $right:expr, $ty:ty) => {{
            let mut pairs = WingParser::parse(<$ty>::RULE, $left).unwrap();
            let val = pairs.next_item::<$ty>().unwrap();
            assert_eq!(val, $right);
            // let mut de = PestDeserializer::parse(<$ty>::RULE, true, $left);
            // let obj = de.deserialize::<$ty>().unwrap();
            // assert_eq!(obj, val);
        }};
    }

    macro_rules! svec {
        ($($arg:expr),* $(,)?) => {
            vec![
                $(Spanned::new_unspanned($arg.into())),*
            ]
        };
    }

    #[test]
    fn test_simple_person() {
        assert_parse!(
            "
                struct Person {
                    age: u8;
                    name: string;
                    mood: f32;
                    hair: Hair,
                }
            ",
            Document {
                user_types: svec![Struct {
                    name: s("Person"),
                    fields: svec![
                        StructField::new("age", Builtin::U8),
                        StructField::new("name", Builtin::String),
                        StructField::new("mood", Builtin::F32),
                        StructField::new("hair", "Hair"),
                    ]
                }]
            }
        )
    }

    #[test]
    fn test_mixed_person() {
        // Please don't do this.
        assert_parse!(
            "
                struct Person {
                    age: u8,
                    name: string,
                    mood: f32,
                    hair: Hair;
                }
            ",
            Document {
                user_types: svec![Struct {
                    name: s("Person"),
                    fields: svec![
                        StructField::new("age", Builtin::U8),
                        StructField::new("name", Builtin::String),
                        StructField::new("mood", Builtin::F32),
                        StructField::new("hair", "Hair"),
                    ]
                }]
            }
        )
    }

    #[test]
    fn test_multiple_structs() {
        // Please don't do this.
        assert_parse!(
            "
                struct A1 {
                    darega: u32;
                    omaga: i32;
                    odiga: f32;
                }

                struct A2 {
                    lerolero: u8;
                    lepolepo: int;
                    tibirabirom: usize;
                }
            ",
            Document {
                user_types: svec![
                    Struct {
                        name: s("A1"),
                        fields: svec![
                            StructField::new("darega", Builtin::U32),
                            StructField::new("omaga", Builtin::I32),
                            StructField::new("odiga", Builtin::F32),
                        ]
                    },
                    Struct {
                        name: s("A2"),
                        fields: svec![
                            StructField::new("lerolero", Builtin::U8),
                            StructField::new("lepolepo", Builtin::Int),
                            StructField::new("tibirabirom", Builtin::USize),
                        ]
                    }
                ]
            }
        )
    }

    #[test]
    fn parse_list() {
        assert_parse!("[int]", Type::list(Builtin::Int), Type);
        assert_parse!("[u8]", Type::list(Builtin::U8), Type);
        assert_parse!("[string]", Type::list(Builtin::String), Type);
    }

    #[test]
    fn parse_list_nested() {
        assert_parse!("[[int]]", Type::list(Type::list(Builtin::Int)), Type);
    }

    #[test]
    fn test_simple_enum() {
        assert_parse!(
            "
                enum Color {
                    RGB: RGB,
                    HSLV: HSLV,
                    Gray: Gray,
                }
            ",
            Document {
                user_types: svec![Enum {
                    name: s("Color"),
                    definitions: svec![
                        StructField::new("RGB", "RGB"),
                        StructField::new("HSLV", "HSLV"),
                        StructField::new("Gray", "Gray"),
                    ]
                }]
            }
        )
    }

    impl EnumVariant {
        fn user_type(val: impl Into<UserType>) -> Self {
            Self::UserType(val.into())
        }
    }

    #[test]
    fn test_enum_composite() {
        assert_parse!(
            "
                enum Message {
                    struct Ping {
                        val: string,
                        code: u32
                    }
                    enum Download {
                        struct Covers;
                        struct Images {
                            by: String
                        }
                    }
                }
            ",
            Document {
                user_types: svec![Enum {
                    name: s("Message"),
                    definitions: svec![
                        EnumVariant::user_type(Struct {
                            name: s("Ping"),
                            fields: svec![
                                StructField::new("val", Builtin::String),
                                StructField::new("code", Builtin::U32),
                            ]
                        }),
                        EnumVariant::user_type(Enum {
                            name: s("Download"),
                            definitions: svec![
                                EnumVariant::user_type(Struct {
                                    name: s("Covers"),
                                    fields: vec![]
                                }),
                                EnumVariant::user_type(Struct {
                                    name: s("Images"),
                                    fields: svec![StructField::new("by", Builtin::String)]
                                })
                            ]
                        })
                    ]
                }]
            }
        )
    }
}
