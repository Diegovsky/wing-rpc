use serde::{Serialize, Deserialize};
use wing_rpc::Message as WingMessage;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Simple {
    pub a: String,
    pub b: u32,
}

impl<'a> WingMessage<'a> for Simple {
    const NAME: &'static str = "Simple";
}

