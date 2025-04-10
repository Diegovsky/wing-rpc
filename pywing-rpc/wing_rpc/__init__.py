from io import BufferedRWPair
from pydantic import BaseModel as Schema
from typing import Protocol


class Writer(Protocol):
    def write(self, data: bytes, /) -> int: ...
    def flush(self) -> None: ...


class Reader(Protocol):
    def read(self, size: int = -1, /) -> bytes: ...


class RW(Writer, Reader): ...


type Stream = RW | BufferedRWPair
