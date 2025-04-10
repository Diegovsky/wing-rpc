from data import Ping, Pong
from wing_rpc.peer import Peer, ClientDisconnectedError
from wing_rpc import Stream
import socket
import time


class Sock:
    def __init__(self, is_server: bool, host="localhost", port=5000):
        self.host = host
        self.port = port
        self.is_server = is_server
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        if is_server:
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            while True:
                try:
                    self.socket.bind((host, port))
                    break
                except OSError:
                    print("Socket in use. Retrying...")
                    time.sleep(0.3)

    def connect(self):
        if self.is_server:
            self.socket.listen(1)
            conn, _ = self.socket.accept()
            return conn.makefile("rbw")
        else:
            self.socket.connect((self.host, self.port))
            return self.socket.makefile("rbw")

    def close(self):
        self.socket.close


def main(sock: Sock):
    try:
        peer = Peer(sock.connect())
        time = 0
        while True:
            if is_server:
                peer.send(Pong(time=time))
                print("sent")
                time = peer.receive(Ping).time + 1
                print("Got:", time)
            else:
                print("receiving")
                time = peer.receive(Pong).time + 1
                print("got")
                peer.send(Ping(time=time))
                print("Got:", time)

            if time == 10:
                break
    except ClientDisconnectedError:
        if sock.is_server:
            main(sock)


if __name__ == "__main__":
    import sys

    args = sys.argv[1:]
    is_server = len(args) > 0 and args[0] == "serve"
    sock = Sock(is_server)

    main(sock)
