use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Void {}

impl std::fmt::Display for Void {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl std::error::Error for Void {}
impl serde::de::Error for Void {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        unreachable!("Error: {}", msg)
    }
}
