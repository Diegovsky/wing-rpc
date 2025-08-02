use serde::{Serialize, Deserialize};
use wing_rpc::Message as WingMessage;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Result {
    pub id: usize,
    pub title: String,
    pub description: String,
}

impl<'a> WingMessage<'a> for Result {
    const NAME: &'static str = "Result";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ByName {
    pub name: String,
}

impl<'a> WingMessage<'a> for ByName {
    const NAME: &'static str = "ByName";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ById {
    pub id: usize,
}

impl<'a> WingMessage<'a> for ById {
    const NAME: &'static str = "ById";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Search {
    ByName(ByName),
    ById(ById),
}

impl<'a> WingMessage<'a> for Search {
    const NAME: &'static str = "Search";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
    Search(ById),
    results(Vec<Result>),
}

impl<'a> WingMessage<'a> for Message {
    const NAME: &'static str = "Message";
}

