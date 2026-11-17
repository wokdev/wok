from io import StringIO

from tests.conftest import expected_config, git

from wok.cmd import init


def test_init_single_repo(repo_sample) -> None:
    sample = repo_sample()
    output = StringIO()
    init(sample.config_path(), sample.repo(), output)
    assert sample.config_path().read_text() == expected_config("empty.toml")
    assert output.getvalue() == f"Created config at `{sample.config_path()}`\n"


def test_init_with_submodules(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"])
    output = StringIO()
    init(sample.config_path(), sample.repo(), output)
    assert sample.config_path().read_text() == expected_config("a-b.toml")
    assert output.getvalue() == f"Created config at `{sample.config_path()}`\n"


def test_init_nested_submodule_path(repo_sample) -> None:
    sample = repo_sample(["deps/sub-a"])
    output = StringIO()
    init(sample.config_path(), sample.repo(), output)
    assert sample.config_path().read_text() == expected_config("deps-a.toml")


def test_init_does_not_switch_branches(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"])
    git(["switch", "-c", "other-branch"], sample.subrepo_path("sub-a"))
    output = StringIO()
    init(sample.config_path(), sample.repo(), output)
    assert sample.config_path().read_text() == expected_config("a-other-b.toml")
    assert (
        git(["branch", "--show-current"], sample.subrepo_path("sub-a")).strip()
        == "other-branch"
    )


def test_init_fails_when_file_exists(repo_sample) -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    sample = repo_sample()
    sample.config_path().write_text(expected_config("empty.toml"))
    result = CliRunner().invoke(app, ["-f", str(sample.config_path()), "init"])
    assert result.exit_code == 1
    assert "already exists" in (result.stderr + result.output)
