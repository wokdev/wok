import pathlib
import typing

import click.testing
import pygit2
import pytest

from wok import cli, config


@pytest.fixture(scope='session')
def repo_1_url(data_dir: pathlib.Path) -> str:
    return str(data_dir / 'repos' / 'prj-1')


@pytest.fixture(scope='session')
def repo_1_path() -> str:
    return './prj-1'


@pytest.fixture(scope='session')
def repo_2_url(data_dir: pathlib.Path) -> str:
    return str(data_dir / 'repos' / 'prj-2')


@pytest.fixture(scope='session')
def repo_2_path() -> str:
    return './prj-2'


@pytest.fixture()
def cli_runner() -> typing.Iterator[click.testing.CliRunner]:
    cli_runner = click.testing.CliRunner()
    with cli_runner.isolated_filesystem():
        yield cli_runner


@pytest.fixture()
def root_repo(cli_runner: click.testing.CliRunner) -> pygit2.Repository:
    cwd = pathlib.Path.cwd()
    cwd.joinpath('readme').write_text('a file to commit')

    root_repo = pygit2.init_repository(path=str(cwd))
    root_repo.index.add_all()
    root_repo.index.write()
    tree = root_repo.index.write_tree()
    root_repo.create_commit(
        'refs/heads/master',
        root_repo.default_signature,
        root_repo.default_signature,
        'Initial commit',
        tree,
        [],
    )
    return root_repo


@pytest.fixture()
def cooked_repo(
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
    repo_1_url: str,
    repo_1_path: str,
    repo_2_url: str,
    repo_2_path: str,
) -> pygit2.Repository:

    assert (
        cli_runner.invoke(cli.main, ['init']).exit_code
        | cli_runner.invoke(cli.main, ['add', repo_1_url, repo_1_path]).exit_code
        | cli_runner.invoke(cli.main, ['add', repo_2_url, repo_2_path]).exit_code
        | cli_runner.invoke(cli.main, ['commit']).exit_code
    ) == 0

    return root_repo


def test_001_init_fails_without_root_repo(
    data_dir: pathlib.Path, cli_runner: click.testing.CliRunner
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 1
    assert result.output.startswith('Unable to find root `wok` repo at')


def test_002_init(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '002_wok.yml'
    )


def test_003_init_in_branch(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
) -> None:
    dev_branch = root_repo.branches.local.create(
        name='dev', commit=next(root_repo.walk(root_repo.head.target))
    )
    root_repo.checkout(refname=dev_branch, strategy=pygit2.GIT_CHECKOUT_FORCE)

    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '003_wok.yml'
    )


def test_011_commit(
    cli_runner: click.testing.CliRunner, root_repo: pygit2.Repository
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['commit'])
    assert result.exit_code == 0, result.output

    assert root_repo.status() == {}


def test_021_add(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
    repo_1_url: str,
    repo_1_path: str,
    repo_2_url: str,
    repo_2_path: str,
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['add', repo_1_url, repo_1_path])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '021_a_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=repo_1_path)
    assert repo_1.remotes['origin'].url == repo_1_url

    result = cli_runner.invoke(cli.main, ['add', repo_2_url, repo_2_path])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '021_b_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_2 = pygit2.Repository(path=repo_2_path)
    assert repo_2.remotes['origin'].url == repo_2_url


def test_022_add_fails_on_already_added(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
    repo_1_url: str,
    repo_1_path: str,
) -> None:
    result = cli_runner.invoke(cli.main, ['add', repo_1_url, repo_1_path])
    assert result.exit_code == 1, result.output
    assert "already congigured" in result.output

    result = cli_runner.invoke(cli.main, ['add', repo_1_url, 'other_path'])
    assert result.exit_code == 1, result.output
    assert "already congigured" in result.output

    result = cli_runner.invoke(cli.main, ['add', 'other_url', repo_1_path])
    assert result.exit_code == 1, result.output
    assert "already congigured" in result.output


def test_031_start(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '031_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    assert cooked_repo.head.shorthand == 'branch-1'


def test_041_join(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
    repo_1_path: str,
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', repo_1_path])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '041_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=repo_1_path)
    assert repo_1.head.shorthand == 'branch-1'


def test_042_join_many(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
    repo_1_path: str,
    repo_2_path: str,
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', repo_1_path, repo_2_path])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '042_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=repo_1_path)
    repo_2 = pygit2.Repository(path=repo_2_path)
    assert repo_1.head.shorthand == 'branch-1'
    assert repo_2.head.shorthand == 'branch-1'


def test_043_join_fails_on_unknown_path(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    cooked_repo: pygit2.Repository,
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    result = cli_runner.invoke(cli.main, ['join', 'unknown/path'])
    assert result.exit_code == 1, result.output
    assert result.output == "Unknown repo path `unknown/path`\n"
