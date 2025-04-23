from wing_rpc.peer import NoMessages, Peer
from wing_rpc.sock import TcpSocket
from client.ipc import Login, ChatEvent, Message, SendMessage, UserJoined, UserLeft

from prompt_toolkit import PromptSession
from prompt_toolkit.patch_stdout import patch_stdout
from prompt_toolkit.application import run_in_terminal
from prompt_toolkit.formatted_text import ANSI
from prompt_toolkit.styles import Style

import threading
from queue import Queue, Empty


# Output display
def message_recv(peer: Peer, msg_queue: Queue[str]):
    while True:
        msg = peer.receive(ChatEvent)
        match msg.value:
            case Message(username, contents):
                msg_queue.put_nowait(f"[{username}]: {contents}")
                pass
            case UserJoined(username):
                msg_queue.put_nowait(f"[user {username}] joined the channel")
            case UserLeft(username):
                msg_queue.put_nowait(f"[user {username}] left the channel")


def main():
    sock = TcpSocket(is_server=False, host="localhost", port=6001)
    peer = sock.connect()
    peer.send(Login(username="user"))

    msg_queue = Queue()
    thread = threading.Thread(
        target=message_recv,
        args=(
            peer,
            msg_queue,
        ),
        daemon=True,
    )
    thread.start()
    # Let's build our window then
    session = PromptSession("> ")

    with patch_stdout():
        while True:
            try:
                while True:
                    msg = msg_queue.get_nowait()
                    run_in_terminal(lambda: print(msg))
            except Empty:
                pass

            try:
                msg = session.prompt()
            except (EOFError, KeyboardInterrupt):
                break

            if msg.strip().lower() in ("/exit", "/quit"):
                break

            peer.send(SendMessage(contents=msg))
