use crate::parser::*;
use pest::iterators::{Pair, Pairs};
use serde::{
    Deserialize,
    de::{Deserializer, EnumAccess, SeqAccess, Visitor},
};
use simplede::DeAdapter;
use span::Span;
/* Possible supported cases of deserialization:
    atom: (ident)
    composite: (struct ident struct_body)
    list: (struct_field a int) (struct_field b f32)
    enum: (struct | enum)
    spanned<T>: (Spanned<ident>)
*/

mod simplede;

type StrArray = &'static [&'static str];

// todo: add size hint
#[derive(Debug, Clone, Copy)]
enum Composite {
    Struct { fields: StrArray },
    Tuple { len: usize },
    Seq,
}

#[derive(Debug, Clone)]
enum Request {
    Atom,
    Enum { variants: StrArray },
    Composite(Composite),
}

impl From<Composite> for Request {
    fn from(value: Composite) -> Self {
        Self::Composite(value)
    }
}

#[derive(Clone)]
struct PestDeserializer<'i> {
    adjust_case: bool,
    pairs: Pairs<'i, Rule>,
}

impl<'de> PestDeserializer<'de> {
    pub(crate) fn parse(rule: Rule, adjust_case: bool, text: &'de str) -> Self {
        Self {
            adjust_case,
            pairs: WingParser::parse2(rule, text).unwrap(),
        }
    }
    fn inner(&mut self) -> Self {
        Self {
            adjust_case: self.adjust_case,
            pairs: self.pairs.next2().into_inner(),
        }
    }
    fn peek(&self) -> Pair<'de, Rule> {
        self.pairs.peek().unwrap()
    }
    fn peek_rule_name(&self) -> String {
        format!("{:?}", self.peek().as_rule())
    }
    fn has_next(&self) -> bool {
        self.pairs.peek().is_some()
    }
}

impl<'de> PestDeserializer<'de> {
    fn d_enum<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Void> {
        self.assert_rule_matches(name);
        self.deserialize_request(Request::Enum { variants }, visitor)
    }

    fn d_seq<V>(&mut self, visitor: V) -> Result<V::Value, Void>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_request(Composite::Seq, visitor)
    }

    fn assert_rule_matches(&self, name: &str) -> Result<(), Void> {
        fn normalize(text: &str) -> String {
            text.replace("_", "").to_lowercase()
        }
        if self.adjust_case {
            assert_eq!(normalize(&*self.peek_rule_name()), normalize(name));
        } else {
            assert_eq!(self.peek_rule_name(), name);
        }
        Ok(())
    }

    fn d_struct<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        fields: StrArray,
        visitor: V,
    ) -> Result<V::Value, Void> {
        if dbg!(name) == "Spanned" {
            return DeAdapter(impls::SpannedDe::new(self.clone()))
                .deserialize_struct(name, fields, visitor);
        }
        self.assert_rule_matches(name)?;
        self.deserialize_request(Composite::Struct { fields }, visitor)
    }
    fn d_tuple_struct<V>(
        &mut self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Void>
    where
        V: serde::de::Visitor<'de>,
    {
        self.assert_rule_matches(name)?;
        self.d_tuple(len, visitor)
    }
    fn d_tuple<V>(&mut self, len: usize, visitor: V) -> Result<<V as Visitor<'de>>::Value, Void>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_request(Composite::Tuple { len }, visitor)
    }
    fn d_newtype<V: Visitor<'de>>(
        &mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Void> {
        self.assert_rule_matches(name)?;
        visitor.visit_newtype_struct(self)
    }
    fn d_string<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value, Void> {
        self.deserialize_request(Request::Atom, visitor)
    }

    fn deserialize_request<V: Visitor<'de>>(
        &mut self,
        request: impl Into<Request>,
        visitor: V,
    ) -> Result<V::Value, Void> {
        match request.into() {
            Request::Atom => visitor.visit_str(self.pairs.next2().as_str()),
            Request::Composite(composite) => visitor.visit_seq(CompositeDe {
                composite,
                de: self.inner(),
            }),
            Request::Enum { .. } => visitor.visit_enum(&mut self.inner()),
        }
    }
    pub fn deserialize<T: Deserialize<'de>>(&mut self) -> Result<T, Void> {
        T::deserialize(self)
    }
}

struct CompositeDe<'de> {
    de: PestDeserializer<'de>,
    composite: Composite,
}

mod err;
mod impls;
pub use err::Void;

#[cfg(test)]
mod test {
    use serde::Deserialize;
    use span::Spanned;

    use super::*;
    use rstest::*;
    use rstest_reuse::*;

    #[template]
    #[rstest]
    #[case::off(false)]
    #[case::on(true)]
    fn adjust_case_toggle(#[case] adjust_case: bool) {}

    #[apply(adjust_case_toggle)]
    fn test_ident(adjust_case: bool) {
        let mut obj = PestDeserializer::parse(Rule::ident, adjust_case, "Olimar");
        assert_eq!("Olimar", obj.deserialize::<String>().unwrap())
    }

    #[apply(adjust_case_toggle)]
    fn test_composite(adjust_case: bool) {
        #[derive(Deserialize, PartialEq)]
        #[serde(rename = "struct")]
        struct Struct {
            ident: String,
        }

        let mut obj = PestDeserializer::parse(Rule::r#struct, adjust_case, "struct example {}");
        let st = obj.deserialize::<Struct>().unwrap();
        assert_eq!("example", st.ident);
    }

    #[apply(adjust_case_toggle)]
    fn test_composite_recursive(adjust_case: bool) {
        #[derive(Deserialize, PartialEq)]
        #[serde(rename = "struct_field")]
        struct StructField {
            name: String,
            ty: String,
        }
        #[derive(Deserialize, PartialEq)]
        #[serde(rename = "struct")]
        struct Struct {
            ident: String,
            body: Vec<StructField>,
        }

        let mut obj = PestDeserializer::parse(
            Rule::r#struct,
            adjust_case,
            "struct Person {
                name: string,
                age: u8,
            }",
        );
        let st = obj.deserialize::<Struct>().unwrap();
        assert_eq!(st.ident, "Person");
        assert_eq!(st.body[0].name, "name");
        assert_eq!(st.body[0].ty, "string");
        assert_eq!(st.body[1].name, "age");
        assert_eq!(st.body[1].ty, "u8");
    }

    #[apply(adjust_case_toggle)]
    fn test_data_enum(adjust_case: bool) {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename = "type")]
        enum Type {
            #[serde(rename = "ident")]
            Ident(String),
            #[serde(rename = "list_type")]
            List(Box<Type>),
        }

        let mut obj = PestDeserializer::parse(Rule::r#type, adjust_case, "u32");
        let st = obj.deserialize::<Type>().unwrap();
        assert_eq!(Type::Ident("u32".into()), st);
    }

    #[apply(adjust_case_toggle)]
    fn test_tuple_composite(adjust_case: bool) {
        let mut obj = PestDeserializer::parse(
            Rule::r#struct_body,
            adjust_case,
            "{ name: string; age: u32 }",
        );
        let fields: Vec<(String, String)> = obj.deserialize().unwrap();
        assert_eq!(fields[0].0, "name");
        assert_eq!(fields[0].1, "string");
        assert_eq!(fields[1].0, "age");
        assert_eq!(fields[1].1, "u32");
    }

    #[apply(adjust_case_toggle)]
    fn test_newtype_struct_composite(adjust_case: bool) {
        #[derive(Deserialize)]
        #[serde(rename = "struct_body")]
        struct StructBody(Vec<(String, String)>);

        let mut obj = PestDeserializer::parse(
            Rule::r#struct_body,
            adjust_case,
            "{ name: string; age: u32 }",
        );
        let fields: StructBody = obj.deserialize().unwrap();
        assert_eq!(fields.0[0].0, "name");
        assert_eq!(fields.0[0].1, "string");
        assert_eq!(fields.0[1].0, "age");
        assert_eq!(fields.0[1].1, "u32");
    }

    #[apply(adjust_case_toggle)]
    fn test_spanned(adjust_case: bool) {
        let mut obj = PestDeserializer::parse(Rule::ident, adjust_case, "Olimar");
        let span: Span = obj.peek().as_span().into();
        let val = obj.deserialize::<Spanned<String>>().unwrap();
        assert_eq!("Olimar", val.value);
        assert_eq!(span, val.span);
    }

    #[apply(adjust_case_toggle)]
    fn test_newtyped_ident(adjust_case: bool) {
        #[derive(Deserialize, PartialEq)]
        #[serde(rename = "ident")]

        struct Ident(String);
        let mut obj = PestDeserializer::parse(Rule::r#ident, adjust_case, "Olimar");
        let st = obj.deserialize::<Ident>().unwrap();
        assert_eq!("Olimar", st.0);
    }

    #[test]
    fn test_mismatched_case() {
        #[derive(Deserialize, PartialEq)]
        struct Ident(String);

        let mut obj = PestDeserializer::parse(Rule::r#ident, true, "Olimar");
        let st = obj.deserialize::<Ident>().unwrap();
        assert_eq!("Olimar", st.0);
    }
}
