from wing_rpc import Schema, Enum
from typing import ClassVar
from enum import StrEnum


class Result(Schema):
    __match_args__: ClassVar[tuple] = ('id', 'title', 'description',)
    id: int
    title: str
    description: str


class ByName(Schema):
    __match_args__: ClassVar[tuple] = ('name',)
    name: str


class ById(Schema):
    __match_args__: ClassVar[tuple] = ('id',)
    id: int


class Search(Enum):
    __match_args__: ClassVar[tuple] = ('tag', 'value',)
    class Tag(StrEnum):
        ByName = 'ByName'
        ById = 'ById'
    tag: Tag
    value: ByName | ById


class Message(Enum):
    __match_args__: ClassVar[tuple] = ('tag', 'value',)
    class Tag(StrEnum):
        Search = 'Search'
        results = 'results'
    tag: Tag
    value: ById | list[Result]


