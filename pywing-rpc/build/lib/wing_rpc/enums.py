from typing import Any, ClassVar
from pydantic import model_validator, model_serializer
from wing_rpc import Schema
import enum


class Enum(Schema):
    tag: Any
    value: Any

    Tag: ClassVar[enum.Enum]

    @model_validator(mode="before")
    @classmethod
    def deserialize_external(cls, data: Any) -> Any:
        assert isinstance(data, dict)
        assert len(data) == 1
        tag = tuple(data.keys())[0]
        return {"value": data[tag], "tag": tag}

    @model_serializer(mode="plain")
    def serialize_external(self) -> Any:
        return {self.tag: self.value}
