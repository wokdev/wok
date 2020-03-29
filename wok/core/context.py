import functools
import pathlib
import sys
import typing

import attr
import click
import pygit2

from wok import config


@attr.s(auto_attribs=True, kw_only=True)
class Context:
    conf_path: pathlib.Path = attr.ib(default=pathlib.Path('wok.yml'))
    _conf: typing.Optional[config.Config] = attr.ib(init=False)
    _root_repo: pygit2.Repository = attr.ib(init=False)

    def __attrs_post_init__(self) -> None:
        cwd = pathlib.Path.cwd()
        root_repo_path = pygit2.discover_repository(str(cwd))
        if root_repo_path is None:
            print(
                f"Unable to find root `wok` repo at `{cwd.absolute()}``",
                file=sys.stderr,
            )
            raise FileNotFoundError(cwd.absolute())
        self._root_repo = pygit2.Repository(path=root_repo_path)

        self._conf = (
            config.Config.load(path=self.conf_path) if self.conf_path.exists() else None
        )

    @property
    def conf(self) -> config.Config:
        if self._conf is None:
            raise FileNotFoundError(self.conf_path.absolute())
        return self._conf

    @property
    def root_repo(self) -> pygit2.Repository:
        return self._root_repo


def with_context(func: typing.Callable) -> typing.Callable:
    @functools.wraps(func)
    def wrapper(*args: typing.Any, **kwargs: typing.Any) -> typing.Any:
        ctx: Context = click.get_current_context().obj
        return func(*args, ctx=ctx, **kwargs)

    return wrapper
