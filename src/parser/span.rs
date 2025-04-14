use derive_more::{Deref, DerefMut};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
}

#[derive(Deref, DerefMut, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    span: Span,
    #[deref_mut]
    #[deref]
    value: T,
}
