from wing_rpc.peer import Peer
import socket


class TcpSocket:
    """This class provides a lightweight abstraction over TCP sockets to get code up and running quickly."""

    def __init__(self, *, is_server: bool, host="localhost", port=6000):
        self.host = host
        self.port = port
        self.is_server = is_server
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        if is_server:
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self.socket.bind((host, port))

    def connect(self, *, blocking=True) -> Peer:
        """Connects to a peer in the configured location.

        Server:
            Blocks the thread waiting for a client to connect.
        Client:
            Initiates a connection to the server.
        """
        print("Connecting to", self.host, self.port)
        if self.is_server:
            conn, _ = self.socket.accept()
            if not blocking:
                conn.setblocking(0)
            stream = conn.makefile("rbw")
        else:
            self.socket.connect((self.host, self.port))
            if not blocking:
                self.socket.setblocking(0)
            stream = self.socket.makefile("rbw")

        return Peer(stream)

    def close(self):
        """Closes the underlying socket."""
        self.socket.close()
