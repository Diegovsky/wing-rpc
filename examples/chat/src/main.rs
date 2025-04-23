use std::{
    collections::VecDeque,
    io::ErrorKind,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use wing_rpc::{
    Peer, Timeout, WingResult,
    server::{Mode, TcpServer},
};

mod ipc;
use ipc::{ChatEvent, Login, Message, SendMessage, UserJoined};

struct Client {
    peer: Peer,
    username: String,
}

fn main() -> WingResult<()> {
    let addr = "0.0.0.0:6001";
    let server = TcpServer::bind(addr)?;
    let mut messages = VecDeque::new();
    let mut clients = vec![];
    loop {
        if clients.is_empty() {
            println!("Waiting for connections...");
            server.listener().set_nonblocking(false)?;
        } else {
            server.listener().set_nonblocking(true)?;
        }
        if let Ok(mut peer) = server.accept(Mode::NonBlocking) {
            let Login { username } = peer.recv()?;
            messages.push_back(ChatEvent::UserJoined(UserJoined {
                username: username.clone(),
            }));
            clients.push(Client { username, peer });
        }
        clients.retain_mut(|client| {
            // try to receive a message, if not available, skip to the next one
            let contents = match client.peer.try_recv(Timeout::DontBlock) {
                Ok(Some(SendMessage { contents })) => contents,
                Ok(None) => return true,
                Err(wing_rpc::Error::Io(e)) if e.kind() == ErrorKind::ConnectionReset => {
                    return false;
                }
                Err(e) => panic!("Error: {e}"),
            };
            messages.push_back(ChatEvent::Message(Message {
                username: client.username.clone(),
                contents,
            }));
            true
        });
        for message in messages.drain(..) {
            for client in &mut clients {
                client.peer.send(message.clone())?;
            }
        }
        if clients.is_empty() {
            break;
        }
    }
    Ok(())
}
