use std::ops::Range;

use derive_more::{AsRef, Deref, DerefMut};
use miette::{SourceOffset, SourceSpan};

use super::{Rule, rules::ParseItem};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
}

impl Span {
    pub fn end(&self) -> usize {
        self.offset + self.length
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.offset..value.end()
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(value: Span) -> Self {
        SourceSpan::new(SourceOffset::from(value.offset), value.length)
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span<'_>) -> Self {
        let start = value.start();
        Self {
            offset: start,
            length: value.end() - start,
        }
    }
}

#[derive(Deref, DerefMut, AsRef, Debug, Clone, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    #[deref_mut]
    #[deref]
    #[as_ref]
    pub value: T,
}

impl<T: std::fmt::Display> std::fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Spanned<T> {
    #[cfg(test)]
    pub fn new_unspanned(value: T) -> Self {
        Self {
            value,
            span: Span::default(),
        }
    }
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            value: map(self.value),
            span: self.span,
        }
    }
    pub fn as_ref<'a>(&'a self) -> Spanned<&'a T> {
        Spanned {
            span: self.span,
            value: &self.value,
        }
    }
}

impl<T> Spanned<Option<T>> {
    pub fn transpose(self) -> Option<Spanned<T>> {
        Some(Spanned {
            span: self.span,
            value: self.value?,
        })
    }
}

impl<T: ParseItem> ParseItem for Spanned<T> {
    const RULE: Rule = T::RULE;

    fn parse<'i>(pair: pest::iterators::Pair<'i, Rule>) -> miette::Result<Self> {
        Ok(Self {
            span: pair.as_span().into(),
            value: T::parse(pair)?,
        })
    }

    fn match_rule<'i, 't>(pair: &pest::iterators::Pair<'i, Rule>) -> miette::Result<()> {
        T::match_rule(pair)
    }
}

pub type S<T> = Spanned<T>;
pub type SVec<T> = Vec<S<T>>;
