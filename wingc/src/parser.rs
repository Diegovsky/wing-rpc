use derive_more::From;
use pest::{
    Parser, RuleType,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use span::Spanned;
use strum::{EnumString, IntoStaticStr};

#[derive(Parser, Debug, Clone, PartialEq)]
#[grammar = "./idl.pest"]
struct WingParser;

mod rules;
pub mod span;
use rules::ParseItem;
pub use span::{S, SVec};

#[easy_ext::ext]
impl<R: RuleType + Send + Sync + 'static, P: Parser<R>> P {
    fn parse2(rule: R, input: &str) -> miette::Result<Pairs<'_, R>> {
        Self::parse(rule, input)
            .map_err(pest::error::Error::into_miette)
            .map_err(miette::Report::from)
    }
}

#[easy_ext::ext]
impl<'i> Pairs<'i, Rule> {
    fn next_item<P: ParseItem>(&mut self) -> miette::Result<P> {
        P::parse(self.next2())
    }
    fn collect_items<P: ParseItem>(&mut self) -> miette::Result<Vec<P>> {
        self.map(|pair| P::parse(pair)).collect()
    }
    fn next2(&mut self) -> Pair<'i, Rule> {
        self.next().unwrap()
    }
}
pub fn parse_document(doc: &str) -> miette::Result<Document> {
    let mut tokens = WingParser::parse2(Rule::document, doc)?;
    // ParseItem assumes is has been given a parent tree.
    // Despite Document being the root item, it needs a fake root to attach itself onto.
    let doc = tokens.next_item()?;
    Ok(doc)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum AtomicType {
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Type {
    Builtin(AtomicType),
    List(Box<Type>),
    User(String),
}

impl Type {
    pub fn get_variant_name(&self) -> String {
        match self {
            Type::User(name) => name.to_string(),
            Type::List(inner) => format!("ListOf{}", inner.get_variant_name()),
            Type::Builtin(tp) => format!("{:?}", tp),
        }
    }
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

pub type EnumVariant = StructField;

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub variants: SVec<EnumVariant>,
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
    pub fn children<'a>(&'a self) -> impl Iterator<Item = S<&'a Type>> {
        match self {
            Self::Struct(st) => st.fields.iter(),
            Self::Enum(en) => en.variants.iter(),
        }
        .map(|fd| fd.as_ref().map(|fd| &fd.typ))
    }
    pub fn is_empty(&self) -> bool {
        match self {
            UserType::Struct(st) => st.fields.is_empty(),
            UserType::Enum(en) => en.variants.is_empty(),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::span::Spanned;
    use super::*;

    impl From<AtomicType> for Type {
        fn from(val: AtomicType) -> Self {
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
            assert_eq!(val, $right)
        }};
    }

    macro_rules! list {
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
                user_types: list![Struct {
                    name: s("Person"),
                    fields: list![
                        StructField::new("age", AtomicType::U8),
                        StructField::new("name", AtomicType::String),
                        StructField::new("mood", AtomicType::F32),
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
                user_types: list![Struct {
                    name: s("Person"),
                    fields: list![
                        StructField::new("age", AtomicType::U8),
                        StructField::new("name", AtomicType::String),
                        StructField::new("mood", AtomicType::F32),
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
                user_types: list![
                    Struct {
                        name: s("A1"),
                        fields: list![
                            StructField::new("darega", AtomicType::U32),
                            StructField::new("omaga", AtomicType::I32),
                            StructField::new("odiga", AtomicType::F32),
                        ]
                    },
                    Struct {
                        name: s("A2"),
                        fields: list![
                            StructField::new("lerolero", AtomicType::U8),
                            StructField::new("lepolepo", AtomicType::Int),
                            StructField::new("tibirabirom", AtomicType::USize),
                        ]
                    }
                ]
            }
        )
    }

    #[test]
    fn parse_list() {
        assert_parse!("[int]", Type::list(AtomicType::Int), Type);
        assert_parse!("[u8]", Type::list(AtomicType::U8), Type);
        assert_parse!("[string]", Type::list(AtomicType::String), Type);
    }

    #[test]
    fn parse_list_nested() {
        assert_parse!("[[int]]", Type::list(Type::list(AtomicType::Int)), Type);
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
                user_types: list![Enum {
                    name: s("Color"),
                    variants: list![
                        StructField::new("RGB", "RGB"),
                        StructField::new("HSLV", "HSLV"),
                        StructField::new("Gray", "Gray"),
                    ]
                }]
            }
        )
    }
}
