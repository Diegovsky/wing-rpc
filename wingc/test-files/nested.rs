use serde::{Serialize, Deserialize};
use wing_rpc::Message as WingMessage;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
}

impl<'a> WingMessage<'a> for Address {
    const NAME: &'static str = "Address";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub address: Address,
}

impl<'a> WingMessage<'a> for Person {
    const NAME: &'static str = "Person";
}

