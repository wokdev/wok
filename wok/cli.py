import pathlib

import click

from . import core


@click.group(name='wok')
@click.pass_context
def main(ctx: click.Context) -> None:
    ctx.obj = core.Context()


@main.command()
def init() -> None:
    core.init()


@main.command()
@click.argument('url')
@click.argument('path', type=click.Path(exists=False))
def add(url: str, path: str) -> None:
    # TODO: Allow to add existing repos
    core.add(path=pathlib.Path(path), url=url)
