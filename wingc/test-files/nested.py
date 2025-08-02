from wing_rpc import Schema, Enum
from typing import ClassVar
from enum import StrEnum


class Address(Schema):
    pass


class Person(Schema):
    __match_args__: ClassVar[tuple] = ('name', 'age', 'address',)
    name: str
    age: int
    address: Address


