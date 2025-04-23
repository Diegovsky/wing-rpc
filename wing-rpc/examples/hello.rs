use serde::{Deserialize, Serialize};
use wing_rpc::{
    WingResult, client,
    server::{Mode, TcpServer},
};

#[derive(Serialize, Deserialize)]
pub struct Hello {
    name: String,
}
impl wing_rpc::Message<'_> for Hello {
    const NAME: &'static str = "Hello";
}

fn main() -> WingResult<()> {
    let addr = "localhost:6000";
    let is_client = std::env::args().nth(1).as_deref() == Some("connect");
    if is_client {
        println!("Starting client...");
        let mut peer = client::tcp(addr)?;
        peer.send(Hello {
            name: "Server".into(),
        })?;
        let hello = peer.recv::<Hello>()?;
        println!("[server] Hello, {}", hello.name);
    } else {
        let server = TcpServer::bind(addr)?;
        loop {
            println!("Waiting for client...");
            let mut peer = server.accept(Mode::Blocking)?;
            let hello = peer.recv::<Hello>()?;
            println!("[client] Hello, {}", hello.name);
            peer.send(Hello {
                name: "Client".into(),
            })?;
        }
    }
    Ok(())
}
