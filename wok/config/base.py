import pathlib
import typing

import attr
import yaml


def _path_converter(path: typing.Union[str, pathlib.Path]) -> pathlib.Path:
    if isinstance(path, str):
        path = pathlib.Path(path)
    return path.absolute().relative_to(pathlib.Path.cwd())


@attr.s(auto_attribs=True, kw_only=True)
class Repo:
    url: str
    path: pathlib.Path = attr.ib(converter=_path_converter)
    ref: str


@attr.s(auto_attribs=True, kw_only=True, eq=False)
class Config:
    version: str = '1.0'
    repos: typing.MutableSequence[Repo] = attr.ib(factory=list)
    _path: pathlib.Path = attr.ib(init=False)

    def save(self) -> None:
        from . import schema

        yaml.safe_dump(
            data=schema.config.dump(obj=self),
            stream=self._path.open('w'),
            sort_keys=False,
        )

    def __eq__(self, other: typing.Union['Config', typing.Any]) -> bool:
        if self.__class__ is not other.__class__:
            return NotImplemented

        print('**********')
        print(self.version, self.repos, other.version, other.repos)
        return (self.version, self.repos) == (other.version, other.repos)

    @classmethod
    def create(cls, path: pathlib.Path) -> None:
        if path.exists():
            raise FileExistsError(path.absolute())

        conf = cls()
        conf._path = path
        conf.save()

    @classmethod
    def load(self, path: pathlib.Path) -> 'Config':
        from . import schema

        if not path.exists():
            raise FileNotFoundError(path.absolute())

        conf: Config = schema.config.load(data=yaml.safe_load(stream=path.open()))
        conf._path = path

        return conf
