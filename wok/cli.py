import click

from . import config


@click.group(name='wok')
def main() -> None:
    pass


@main.command()
def init() -> None:
    config.create_config()


@main.command()
def sync() -> None:
    pass
