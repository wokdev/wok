from typer.testing import CliRunner

from wok.cli import app

runner = CliRunner()


def test_completion_bash() -> None:
    result = runner.invoke(app, ["completion", "bash"])
    assert result.exit_code == 0
    assert "_wok_completion" in result.stdout
    assert "complete" in result.stdout


def test_completion_fish() -> None:
    result = runner.invoke(app, ["completion", "fish"])
    assert result.exit_code == 0
    assert "_wok_completion" in result.stdout


def test_completion_zsh() -> None:
    result = runner.invoke(app, ["completion", "zsh"])
    assert result.exit_code == 0
    assert "#compdef wok" in result.stdout


def test_completion_defaults_to_bash() -> None:
    result = runner.invoke(app, ["completion"])
    assert result.exit_code == 0
    assert "_wok_completion" in result.stdout


def test_completion_help_lists_shells() -> None:
    result = runner.invoke(app, ["completion", "--help"])
    assert result.exit_code == 0
    for shell in ("bash", "fish", "zsh", "[default: bash]"):
        assert shell in result.stdout


def test_completion_rejects_unknown_shell() -> None:
    result = runner.invoke(app, ["completion", "powershell"])
    assert result.exit_code != 0
