import pathlib

import click.testing
import pytest

from wok import cli


@pytest.fixture()
def cli_runner() -> click.testing.CliRunner:
    return click.testing.CliRunner()


def test_001_init(data_dir: pathlib.Path, cli_runner: click.testing.CliRunner) -> None:
    with cli_runner.isolated_filesystem():
        result = cli_runner.invoke(cli.main, ['init'])
        assert result.exit_code == 0
        assert (
            pathlib.Path('wok.yml').read_text()
            == data_dir.joinpath('001_wok.yml').read_text()
        )
