import pathlib
import typing

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
def commit() -> None:
    core.commit()


@main.command()
@click.argument('url')
@click.argument('path', type=click.Path(exists=False))
def add(url: str, path: str) -> None:
    # TODO: Allow to `import` existing repos
    core.add(path=pathlib.Path(path), url=url)


@main.command()
@click.argument('branch-name')
def start(branch_name: str) -> None:
    # TODO: Allow to `swicth` to existing branch
    core.start(branch_name=branch_name)


@main.command()
@click.argument('repo-path', nargs=-1)
def join(repo_path: typing.Iterable[str]) -> None:
    core.join(repo_paths=[pathlib.Path(repo_path_item) for repo_path_item in repo_path])


@main.command()
def push() -> None:
    core.push()


@main.command()
@click.argument('finish-message')
def finish(finish_message: str) -> None:
    core.finish(message=finish_message)


@main.command()
@click.argument('tag-name')
def tag(tag_name: str) -> None:
    core.tag(tag_name=tag_name)


@main.command()
def sync() -> None:
    core.sync()
