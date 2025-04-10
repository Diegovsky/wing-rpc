macro_rules! impl_parse_atom {
    ($(
        #[rule($rulevar:ident)]
        fn parse(self: $ty:ty, $inner:ident: Pair<Rule>) -> Self $expr:expr)*) => {
        $(
            impl ParseItem for $ty {
                const RULE: Rule = Rule::$rulevar;
                fn parse(rule: Pair<Rule>) -> miette::Result<$ty> {
                    match &rule.as_rule() {
                        Rule::$rulevar => {
                            #[allow(unused_mut)]
                            let mut $inner = rule;
                            Ok($expr)
                        },
                        _ => unreachable!("Invalid token sequence: {:?}; Expected: {}", rule, stringify!($rulevar))
                    }
                }
            }
        )*
    };
}

macro_rules! impl_parse_composite {
    ($(
        #[rule($rulevar:ident)]
        fn parse(self: $ty:ty, $inner:ident: Pairs<Rule>) -> Self $expr:expr)*) => {
        $(
            impl ParseItem for $ty {
                const RULE: Rule = Rule::$rulevar;
                fn parse(rule: Pair<Rule>) -> miette::Result<$ty> {
                    match &rule.as_rule() {
                        Rule::$rulevar => {
                            #[allow(unused_mut)]
                            let mut $inner = rule.into_inner();
                            Ok($expr)
                        },
                        _ => unreachable!("Invalid token sequence: {:?}; Expected: {}", rule, stringify!($rulevar))
                    }
                }
            }
        )*
    };
}

pub(super) use impl_parse_atom;
pub(super) use impl_parse_composite;
