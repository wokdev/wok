import pathlib
import typing

import _pytest
import click.testing
import pygit2
import pytest

from wok import cli
from wok.core import context


@pytest.fixture()
def cli_runner() -> click.testing.CliRunner:
    return click.testing.CliRunner()


def test_001_init_output(
    cli_runner: click.testing.CliRunner, root_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_011_commit_output(
    cli_runner: click.testing.CliRunner, root_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['commit'])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_021_add_output(
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
    repo_1_path: pathlib.Path,
    repo_1_url: str,
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['add', repo_1_url, str(repo_1_path)])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_022_add_converts_to_path(
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
    repo_1_path: pathlib.Path,
    repo_1_url: str,
    monkeypatch: _pytest.monkeypatch.MonkeyPatch,
) -> None:
    @context.with_context
    def _add(ctx: context.Context, path: pathlib.Path, url: str) -> None:
        assert isinstance(path, pathlib.Path)

    monkeypatch.setattr('wok.core.add', _add)

    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['add', repo_1_url, str(repo_1_path)])
    assert result.exit_code == 0, result.output


def test_031_start_output(
    cli_runner: click.testing.CliRunner, cooked_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_041_join_output(
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
    repo_1_path: pathlib.Path,
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', str(repo_1_path)])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_042_join_converts_to_path(
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
    repo_1_path: pathlib.Path,
    monkeypatch: _pytest.monkeypatch.MonkeyPatch,
) -> None:
    @context.with_context
    def _join(ctx: context.Context, repo_paths: typing.Iterable[pathlib.Path]) -> None:
        assert all(isinstance(repo_path, pathlib.Path) for repo_path in repo_paths)

    monkeypatch.setattr('wok.core.join', _join)

    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', str(repo_1_path)])
    assert result.exit_code == 0, result.output


def test_051_push_output(
    cli_runner: click.testing.CliRunner,
    repo_1_path: pathlib.Path,
    tmp_repos: typing.Iterable[pygit2.Repository],
) -> None:
    repo_1, repo_2, cooked_repo = tmp_repos

    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', str(repo_1_path)])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['commit'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['push'])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_061_finish_output(
    cli_runner: click.testing.CliRunner, cooked_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['finish', "Finishing message"])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_071_tag_output(
    cli_runner: click.testing.CliRunner, cooked_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['tag', 'test-tag'])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_081_sync_output(
    cli_runner: click.testing.CliRunner,
    repo_1_path: pathlib.Path,
    tmp_repos: typing.Iterable[pygit2.Repository],
) -> None:
    repo_1, repo_2, cooked_repo = tmp_repos
    branch_name = 'branch-1'

    result = cli_runner.invoke(cli.main, ['start', branch_name])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', str(repo_1_path)])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['commit'])
    assert result.exit_code == 0, result.output

    cooked_repo.checkout(refname=cooked_repo.lookup_reference_dwim('master'))

    result = cli_runner.invoke(cli.main, ['sync'])
    assert result.exit_code == 0, result.output

    cooked_repo.checkout(refname=cooked_repo.lookup_reference_dwim(branch_name))

    result = cli_runner.invoke(cli.main, ['sync'])
    assert result.exit_code == 0, result.output

    assert result.output == ''


def test_091_fork_output(
    cli_runner: click.testing.CliRunner, cooked_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['fork', 'branch-1'])
    assert result.exit_code == 0, result.output

    assert result.output == ''
