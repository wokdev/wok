import pathlib

import attr
import click

from . import config, core


@attr.s(auto_attribs=True, kw_only=True)
class Context:
    config_path: pathlib.Path = attr.ib(default=pathlib.Path('wok.yml'))


@click.group(name='wok')
@click.pass_context
def main(ctx: click.Context) -> None:
    ctx.obj = Context()


@main.command()
@click.pass_obj
def init(ctx: Context) -> None:
    config.Config.create(path=ctx.config_path)


@main.command()
@click.pass_obj
@click.argument('url')
@click.argument('path', type=click.Path(exists=False))
def add(ctx: Context, url: str, path: str) -> None:
    # TODO: Allow to add existing repos
    core.add(
        conf=config.Config.load(path=ctx.config_path), path=pathlib.Path(path), url=url
    )
