from typer.testing import CliRunner

from wok import __version__
from wok.cli import app

runner = CliRunner()


def test_version_flag() -> None:
    result = runner.invoke(app, ["--version"])
    assert result.exit_code == 0
    assert "wok" in result.stdout
    assert __version__ in result.stdout


def test_version_short_flag() -> None:
    result = runner.invoke(app, ["-V"])
    assert result.exit_code == 0
    assert "wok" in result.stdout
    assert __version__ in result.stdout
