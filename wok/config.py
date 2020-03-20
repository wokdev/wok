import pathlib

import marshmallow as ma
import yaml


class BaseSchema(ma.Schema):
    class Meta:
        unknown = ma.RAISE


class RepoSchema(BaseSchema):
    url = ma.fields.String(required=True)
    path = ma.fields.String(required=True)
    branch = ma.fields.String(missing='master', default='master')


class ConfigSchema(BaseSchema):
    version = ma.fields.String(required=True, default='1.0')
    repos = ma.fields.Nested(RepoSchema, many=True, default=lambda: list())


config_schema = ConfigSchema()


def create_config(config_path: pathlib.Path = pathlib.Path('wok.yml')) -> None:
    if config_path.exists():
        raise FileExistsError(config_path.absolute())
    yaml.safe_dump(
        data=config_schema.dump({}), stream=config_path.open('w'), sort_keys=False
    )
