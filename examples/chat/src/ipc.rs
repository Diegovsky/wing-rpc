use serde::{Serialize, Deserialize};
use wing_rpc::Message as WingMessage;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub username: String,
    pub contents: String,
}

impl<'a> WingMessage<'a> for Message {
    const NAME: &'static str = "Message";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserJoined {
    pub username: String,
}

impl<'a> WingMessage<'a> for UserJoined {
    const NAME: &'static str = "UserJoined";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLeft {
    pub username: String,
}

impl<'a> WingMessage<'a> for UserLeft {
    const NAME: &'static str = "UserLeft";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatEvent {
    Message(Message),
    UserJoined(UserJoined),
    UserLeft(UserLeft),
}

impl<'a> WingMessage<'a> for ChatEvent {
    const NAME: &'static str = "ChatEvent";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
}

impl<'a> WingMessage<'a> for Login {
    const NAME: &'static str = "Login";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessage {
    pub contents: String,
}

impl<'a> WingMessage<'a> for SendMessage {
    const NAME: &'static str = "SendMessage";
}

