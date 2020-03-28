import typing

import marshmallow as ma

from . import base


class Schema(ma.Schema):
    class Meta:
        unknown = ma.RAISE


class Repo(Schema):
    url = ma.fields.String(required=True)
    path = ma.fields.String(required=True)
    ref = ma.fields.String(missing='master')

    @ma.post_load
    def make(self, data: typing.Mapping, **kwargs: typing.Any) -> base.Repo:
        return base.Repo(**data)


class Config(Schema):
    version = ma.fields.String(required=True)
    ref = ma.fields.String(required=True)
    repos = ma.fields.Nested(Repo, many=True)

    @ma.post_load
    def make(self, data: typing.Mapping, **kwargs: typing.Any) -> base.Config:
        return base.Config(**data)


config = Config()
