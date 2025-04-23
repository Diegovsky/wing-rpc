from io import BufferedRWPair
from pydantic import BaseModel as Schema
from wing_rpc.enums import Enum
from typing import Protocol


class Writer(Protocol):
    """A protocol for an object that writes to a resouce, such as a file, socket, etc."""

    def write(self, data: bytes, /) -> int: ...
    def flush(self) -> None: ...


class Reader(Protocol):
    """A protocol for an object that reads from a resouce, such as a file, socket, etc."""

    def read(self, size: int = -1, /) -> bytes: ...


class RW(Writer, Reader):
    """A protocol for an object that both writes and reads from a resouce."""


type Stream = RW | BufferedRWPair

__all__ = ["Schema", "Enum", "Writer", "Reader", "RW", "Stream"]
