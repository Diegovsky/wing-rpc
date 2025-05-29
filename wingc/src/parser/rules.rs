use super::*;
use macro_rules_attribute::apply;

macro_rules! impl_parse_composite {
    (
        $(#[ignore($spantree:ident)])?
        #[rule($rulevar:ident)]
        fn parse($inner:ident: Pairs<Rule>) -> $ty:ty $expr:block) => {
        impl ParseItem for $ty {
            const RULE: Rule = Rule::$rulevar;
            fn parse<'i>(pair: Pair<'i, Rule>) -> miette::Result<Self> {
                let mut $inner = pair.into_inner();
                #[allow(unused_mut)]
                Ok($expr)
            }
        }
    };
}

fn match_rule<'i, 't>(rule: Rule, pair: &Pair<'i, Rule>) -> miette::Result<()> {
    if rule == pair.as_rule() {
        Ok(())
    } else {
        unreachable!("Invalid token sequence: {:?};\nExpected: {:?}", pair, rule)
    }
}
pub trait ParseItem: Sized {
    const RULE: Rule;
    fn parse<'i>(pair: Pair<'i, Rule>) -> miette::Result<Self>;
    fn match_rule<'i, 't>(pair: &Pair<'i, Rule>) -> miette::Result<()> {
        match_rule(Self::RULE, pair)
    }
}

impl ParseItem for String {
    const RULE: Rule = Rule::ident;
    fn parse<'i>(pair: Pair<'i, Rule>) -> miette::Result<Self> {
        Self::match_rule(&pair)?;
        Ok(pair.as_str().to_owned())
    }
}

#[apply(impl_parse_composite)]
#[rule(r#type)]
fn parse(pairs: Pairs<Rule>) -> Type {
    let tk = dbg!(pairs.peek().unwrap());
    if tk.as_rule() == Rule::r#type {
        pairs.next_item::<Self>()?
    } else if tk.as_rule() == Rule::list_type {
        Self::List(Box::new(pairs.next_item()?))
    } else {
        let tname = pairs.next2().as_str();
        tname
            .to_lowercase()
            .parse::<Builtin>()
            .map(Type::Builtin)
            .unwrap_or_else(|_| Type::User(tname.to_owned()))
    }
}

#[apply(impl_parse_composite)]
#[ignore(spantree)]
#[rule(struct_field)]
fn parse(pairs: Pairs<Rule>) -> StructField {
    StructField {
        name: pairs.next_item()?,
        typ: pairs.next_item()?,
    }
}
#[apply(impl_parse_composite)]
#[rule(struct_body)]
fn parse(pairs: Pairs<Rule>) -> SVec<StructField> {
    pairs.collect_items()?
}
#[apply(impl_parse_composite)]
#[rule(r#struct)]
fn parse(pairs: Pairs<Rule>) -> Struct {
    Struct {
        name: pairs.next_item()?,
        fields: pairs.next_item()?,
    }
}
#[apply(impl_parse_composite)]
#[rule(document)]
fn parse(pairs: Pairs<Rule>) -> Document {
    Document {
        user_types: pairs.collect_items()?,
    }
}

#[apply(impl_parse_composite)]
#[rule(enum_variant)]
fn parse(pairs: Pairs<Rule>) -> EnumVariant {
    let inner = pairs.next2();
    if inner.as_rule() == Rule::struct_field {
        EnumVariant::NamedVariant(ParseItem::parse(inner)?)
    } else {
        EnumVariant::UserType(ParseItem::parse(inner)?)
    }
}

#[apply(impl_parse_composite)]
#[rule(enum_body)]
fn parse(pairs: Pairs<Rule>) -> SVec<EnumVariant> {
    pairs.collect_items()?
}

#[apply(impl_parse_composite)]
#[rule(r#enum)]
fn parse(pairs: Pairs<Rule>) -> Enum {
    Enum {
        name: pairs.next_item()?,
        definitions: pairs.next_item()?,
    }
}

#[apply(impl_parse_composite)]
#[ignore(spantree)]
#[rule(user_type)]
fn parse(pairs: Pairs<Rule>) -> UserType {
    let inner = pairs.next2();
    if inner.as_rule() == Rule::r#struct {
        UserType::Struct(ParseItem::parse(inner)?)
    } else {
        UserType::Enum(ParseItem::parse(inner)?)
    }
}
