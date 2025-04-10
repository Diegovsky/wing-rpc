use pest::{
    Parser, RuleType,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use strum::EnumString;

#[derive(Parser, Debug, Clone, PartialEq)]
#[grammar = "./idl.pest"]
struct WingParser;

mod macros;
use macros::*;

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
    Ok(tokens.next_item()?)
}

#[derive(Debug, Clone, PartialEq, EnumString)]
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
    String,
    Binary,
}

trait ParseItem: Sized {
    const RULE: Rule;
    fn parse(rule: Pair<Rule>) -> miette::Result<Self>;
}

impl_parse_atom! {
    #[rule(ident)]
    fn parse(self: String, pair: Pair<Rule>) -> Self {
        pair.as_str().to_string()
    }
}

impl_parse_composite! {
    #[rule(r#type)]
    fn parse(self: Type, pairs: Pairs<Rule>) -> Self {
        if pairs.peek()
            .map(|pair| pair.as_rule() == Self::RULE)
            .unwrap_or(false) {
            Self::List(Box::new(pairs.next_item()?))
        } else {
            let tname = pairs.next2().as_str();
            tname.parse::<AtomicType>()
                .map(Type::Builtin)
                .unwrap_or_else(|_| Type::User(tname.to_owned()))
        }
    }

    #[rule(struct_field)]
    fn parse(self: StructField, pairs: Pairs<Rule>) -> Self {
        StructField {
            name: pairs.next_item()?,
            type_: pairs.next_item()?,
        }
    }
    #[rule(struct_body)]
    fn parse(self: Vec<StructField>, pairs: Pairs<Rule>) -> Self {
        pairs.collect_items()?
    }
    #[rule(r#struct)]
    fn parse(self: Struct, pairs: Pairs<Rule>) -> Self {
        Struct {
            name: pairs.next_item()?,
            fields: pairs.next_item()?,
        }
    }
    #[rule(document)]
    fn parse(self: Document, pairs: Pairs<Rule>) -> Self {
        Document {
            user_types: pairs.collect_items()?
        }
    }
    #[rule(r#enum)]
    fn parse(self: Enum, pairs: Pairs<Rule>) -> Self {

    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Builtin(AtomicType),
    List(Box<Type>),
    User(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub user_types: Vec<Struct>,
}

#[cfg(test)]
pub mod test {
    use super::*;

    impl StructField {
        fn new(name: &str, type_: impl Into<Type>) -> Self {
            Self {
                name: s(name),
                type_: type_.into(),
            }
        }
    }

    impl From<AtomicType> for Type {
        fn from(val: AtomicType) -> Self {
            Type::Builtin(val)
        }
    }

    impl From<&str> for Type {
        fn from(val: &str) -> Self {
            Type::User(s(val))
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
            assert_eq!(parse_document($left).unwrap(), $right)
        };

        ($left:expr, $right:expr, $ty:ty) => {
            let mut pairs = WingParser::parse(<$ty>::RULE, $left).unwrap();
            let val: $ty = pairs.next_item().unwrap();
            assert_eq!(val, $right)
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
                user_types: vec![Struct {
                    name: s("Person"),
                    fields: vec![
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
                user_types: vec![Struct {
                    name: s("Person"),
                    fields: vec![
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
                user_types: vec![
                    Struct {
                        name: s("A1"),
                        fields: vec![
                            StructField::new("darega", AtomicType::U32),
                            StructField::new("omaga", AtomicType::I32),
                            StructField::new("odiga", AtomicType::F32),
                        ]
                    },
                    Struct {
                        name: s("A2"),
                        fields: vec![
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
}
