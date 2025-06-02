use serde::de::{IntoDeserializer, value::MapDeserializer};
use simplede::SimpleDe;

use super::*;

pub(crate) fn span_de<'de>(
    span: Span,
) -> MapDeserializer<'de, impl Iterator<Item = (&'static str, usize)>, Void> {
    MapDeserializer::new([("offset", span.offset), ("length", span.length)].into_iter())
        .into_deserializer()
}

pub(crate) struct SpannedDe<'a, 'de> {
    de: &'a mut PestDeserializer<'de>,
    state: State,
}

impl<'a, 'de> SpannedDe<'a, 'de> {
    pub(crate) fn new(de: &'a mut PestDeserializer<'de>) -> Self {
        Self {
            state: State::Span(de.peek().as_span().into()),
            de,
        }
    }
}

enum State {
    Span(Span),
    Value,
    End,
}

impl<'de> SimpleDe<'de> for SpannedDe<'_, 'de> {
    type Error = Void;

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        assert_eq!(name, "Spanned");
        assert_eq!(fields, &["span", "value"]);
        visitor.visit_seq(self)
    }
}

impl<'de> SeqAccess<'de> for SpannedDe<'_, 'de> {
    type Error = Void;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match &mut self.state {
            State::Span(span) => {
                let result = seed.deserialize(span_de(*span)).map(Some);
                self.state = State::Value;
                result
            }
            State::Value => {
                let result = seed.deserialize(&mut *self.de).map(Some);
                self.state = State::End;
                result
            }
            State::End => return Ok(None),
        }
    }
}
