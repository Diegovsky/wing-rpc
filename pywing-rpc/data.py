from pydantic import BaseModel as Schema


class Ping(Schema):
    time: int


class Pong(Schema):
    time: int


