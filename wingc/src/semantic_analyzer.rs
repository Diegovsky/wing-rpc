use std::collections::HashSet;

use miette::{Diagnostic, LabeledSpan};
use nucleo_matcher::{Matcher, pattern::Atom};
use thiserror::Error;

use crate::parser::{Document, S, Type};

type R = miette::Result<()>;

fn join_errors(errs: &[Error]) -> String {
    errs.iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("\n")
}

type Errors = Vec<Error>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Undefined type '{name}'.")]
    UndefinedType {
        name: S<String>,
        suggestion: Option<String>,
    },
    #[error("Repeated variant '{variant}' in enum '{parent}'.")]
    RepeatedEnumVariant { variant: S<Type>, parent: S<String> },
    #[error("Many errors where found.\n{}", join_errors(.0))]
    MultipleErrors(Errors),
}

fn spanned_labels<'a>(
    labels: impl 'a + IntoIterator<Item = S<String>>,
) -> Option<Box<dyn 'a + Iterator<Item = miette::LabeledSpan>>> {
    Some(Box::new(labels.into_iter().map(|label| {
        LabeledSpan::new_with_span(Some(label.value), label.span)
    })))
}

impl Diagnostic for Error {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self {
            Self::UndefinedType {
                name,
                suggestion: Some(sugg),
                ..
            } => spanned_labels([name.as_ref().map(|_| format!("Did you mean '{sugg}'?"))]),
            Self::RepeatedEnumVariant { variant, parent } => spanned_labels([
                variant.as_ref().map(|_| "Here".into()),
                parent.as_ref().map(|_| "In this enum".into()),
            ]),
            _ => None,
        }
    }
    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        match self {
            Self::MultipleErrors(errs) => Some(Box::new(errs.iter().map(|e| e as &dyn Diagnostic))),
            _ => None,
        }
    }
}

impl Error {}

fn fuzzy_match<'a>(
    candidates_iter: impl Clone + Iterator<Item = &'a str>,
    mut matcher: &mut Matcher,
    search: &str,
) -> Option<&'a str> {
    let candidates = candidates_iter.clone().collect::<Vec<_>>();
    let mut candidates = candidates
        .iter()
        .map(|candidate| {
            (
                *candidate,
                rapidfuzz::distance::jaro_winkler::similarity(search.chars(), candidate.chars()),
            )
        })
        .filter(|(_, val)| *val > 0.90)
        .collect::<Vec<_>>();
    if !candidates.is_empty() {
        candidates.sort_by(|(_, val), (_, val2)| val.total_cmp(val2));
        candidates.get(0).map(|(c, _)| *c)
    } else {
        Atom::new(
            search,
            nucleo_matcher::pattern::CaseMatching::Smart,
            nucleo_matcher::pattern::Normalization::Smart,
            nucleo_matcher::pattern::AtomKind::Fuzzy,
            false,
        )
        .match_list(candidates_iter, &mut matcher)
        .get(0)
        .map(|(original, _)| *original)
    }
}

fn analyze_missing_types(document: &Document) -> Errors {
    let known_types: HashSet<_> = document.user_types.iter().map(|ut| ut.name()).collect();
    let mut matcher = Matcher::default();
    document
        .user_types
        .iter()
        .flat_map(|ut| {
            ut.children()
                .filter_map(|tp| tp.map(Type::as_user).transpose())
                .filter(|name| !known_types.contains(name.value))
        })
        .map(|missing| Error::UndefinedType {
            suggestion: fuzzy_match(known_types.iter().copied(), &mut matcher, missing.value)
                .map(ToString::to_string),
            name: missing.map(ToString::to_string),
        })
        .collect()
}

pub fn analyze_errors(document: &Document) -> R {
    let mut errs = analyze_missing_types(document);
    if errs.is_empty() {
        Ok(())
    } else if errs.len() == 1 {
        Err(errs.pop().unwrap().into())
    } else {
        Err(Error::MultipleErrors(errs).into())
    }
}
