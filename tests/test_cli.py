import pathlib
import typing

import click.testing
import pytest

from wok import cli, config


@pytest.fixture()
def cli_runner() -> typing.Iterator[click.testing.CliRunner]:
    cli_runner = click.testing.CliRunner()
    with cli_runner.isolated_filesystem():
        yield cli_runner


def test_001_init(data_dir: pathlib.Path, cli_runner: click.testing.CliRunner) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '001_wok.yml'
    )


def test_002_add(data_dir: pathlib.Path, cli_runner: click.testing.CliRunner) -> None:
    cli_runner.invoke(cli.main, ['init'])

    result = cli_runner.invoke(
        cli.main, ['add', str(data_dir / 'repos' / 'prj-1'), './prj-1']
    )
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))

    expected_config = config.Config.load(path=data_dir / '002_a_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)

    assert actual_config == expected_config

    result = cli_runner.invoke(
        cli.main, ['add', str(data_dir / 'repos' / 'prj-2'), './prj-2']
    )
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))

    expected_config = config.Config.load(path=data_dir / '002_b_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)

    assert actual_config == expected_config
