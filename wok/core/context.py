import functools
import pathlib
import typing

import attr
import click

from wok import config


@attr.s(auto_attribs=True, kw_only=True)
class Context:
    config_path: pathlib.Path = attr.ib(default=pathlib.Path('wok.yml'))


def with_conf_path(func: typing.Callable) -> typing.Callable:
    @functools.wraps(func)
    def wrapper(*args: typing.Any, **kwargs: typing.Any) -> typing.Any:
        ctx: Context = click.get_current_context().obj
        return func(*args, conf_path=ctx.config_path, **kwargs)

    return wrapper


def with_conf(func: typing.Callable) -> typing.Callable:
    @functools.wraps(func)
    @with_conf_path
    def wrapper(
        *args: typing.Any, conf_path: pathlib.Path, **kwargs: typing.Any
    ) -> typing.Any:
        return func(*args, conf=config.Config.load(path=conf_path), **kwargs)

    return wrapper
