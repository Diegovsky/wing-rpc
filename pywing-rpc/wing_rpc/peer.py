from io import BufferedRWPair
from wing_rpc import Schema, Stream
from wing_rpc.protocol import MismatchingMessageException, wrap, unwrap, WireHeader
import json


class ClientDisconnectedError(Exception):
    pass


class NoMessages(Exception):
    pass


class Peer:
    def __init__(self, file: Stream):
        self.file = file

    def send(self, obj: Schema):
        message = wrap(obj)
        message = json.dumps(message).encode()
        header = WireHeader.from_message(message)
        self.file.write(header.encode() + message)
        self.file.flush()

    def _read(self, count: int):
        b = self.file.read(count)
        if b is None:
            raise NoMessages
        if len(b) == 0:
            raise ClientDisconnectedError()
        return b

    def receive[T: Schema](self, cls: type[T]) -> T:
        header = self._read(WireHeader.byte_count())
        header = WireHeader.from_encoded(header)
        encoded_msg = self._read(header.len)
        data = json.loads(encoded_msg)
        return unwrap(cls, data)

    def try_receive[T: Schema](self, cls: type[T]) -> T | None:
        try:
            return self.receive(cls)
        except MismatchingMessageException:
            return None

    def close(self):
        if isinstance(self.file, BufferedRWPair):
            self.file.close()
