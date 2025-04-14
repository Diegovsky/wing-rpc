use std::collections::{HashMap, HashSet};

use miette::{Diagnostic, SourceOffset, SourceSpan};
use nucleo_matcher::{Matcher, pattern::Atom};
use pest::Span;
use thiserror::Error;

use crate::parser::{Document, UserType};

type R = miette::Result<()>;

fn sex(errs: &[Error]) -> String {
    errs.iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("\n")
}

#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Undefined type '{name}'.")]
    UndefinedType {
        name: String,
        #[diagnostic(help("Did you mean '{suggestion}'?"))]
        suggestion: Option<String>,
    },
    #[error("Many errors where found.\n{}", sex(.0))]
    MultipleErrors(#[related] Vec<Error>),
}

// impl Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::UndefinedType { name, .. } => write!("Undefined type '{}'.", name),
//         }
//     }
// }

impl Error {}

fn to_source(span: &Span) -> SourceSpan {
    SourceSpan::new(SourceOffset::from(span.start()), span.end())
}

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
        .filter(|(_, val)| *val > 0.92)
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

fn analyze_missing_types(document: &Document) -> Vec<Error> {
    let known_types: HashSet<_> = document.user_types.iter().map(|ut| ut.name()).collect();
    let mut matcher = Matcher::default();
    document
        .user_types
        .iter()
        .flat_map(|ut| {
            ut.children()
                .filter_map(|tp| tp.as_user())
                .filter(|name| known_types.contains(name))
        })
        .map(|missing| Error::UndefinedType {
            name: missing.to_owned(),
            suggestion: fuzzy_match(known_types.iter().copied(), &mut matcher, missing)
                .map(ToString::to_string),
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
