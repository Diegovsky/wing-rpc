from wing_rpc import Schema, Enum
from typing import ClassVar
from enum import StrEnum


class Simple(Schema):
    __match_args__: ClassVar[tuple] = ('a', 'b',)
    a: str
    b: int


