from typing import Any, Self, ClassVar
from dataclasses import dataclass
from struct import Struct
from wing_rpc import Schema


def _name(obj: Schema | type[Schema]) -> str:
    if isinstance(obj, Schema):
        cls = obj.__class__
    else:
        cls = obj
    return cls.__name__


type Data = dict[str, Any]


def wrap(obj: Schema) -> Data:
    data = obj.model_dump()
    return {"type": _name(obj), "data": data}


def unwrap[T: Schema](cls: type[T], data: Data) -> None | T:
    if data["type"] == _name(cls):
        return cls(**data["data"])
    else:
        return None


@dataclass
class WireHeader:
    len: int
    flags: int

    wire: ClassVar = Struct("<cH")

    @classmethod
    def from_encoded(cls, msg: bytes) -> Self:
        flags, len = cls.wire.unpack_from(msg)
        return cls(flags=flags, len=len)

    def encode(self) -> bytes:
        flags = bytes([self.flags])
        return self.wire.pack(flags, self.len)

    @classmethod
    def from_message(cls, msg: bytes, flags: int = 0) -> Self:
        return cls(flags=flags, len=len(msg))

    @classmethod
    def byte_count(cls) -> int:
        return cls.wire.size
