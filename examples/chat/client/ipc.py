from wing_rpc import Schema, Enum
from typing import ClassVar
from enum import StrEnum


class Message(Schema):
    __match_args__: ClassVar[tuple] = ('username', 'contents',)
    username: str
    contents: str


class UserJoined(Schema):
    __match_args__: ClassVar[tuple] = ('username',)
    username: str


class UserLeft(Schema):
    __match_args__: ClassVar[tuple] = ('username',)
    username: str


class ChatEvent(Enum):
    class Tag(StrEnum):
        Message = 'Message'
        UserJoined = 'UserJoined'
        UserLeft = 'UserLeft'
    tag: Tag
    value: Message | UserJoined | UserLeft


class Login(Schema):
    __match_args__: ClassVar[tuple] = ('username',)
    username: str


class SendMessage(Schema):
    __match_args__: ClassVar[tuple] = ('contents',)
    contents: str


