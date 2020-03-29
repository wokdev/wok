import pathlib
import typing

import click.testing
import pygit2
import pytest

from wok import cli, config


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
def cooked(data_dir: pathlib.Path, cli_runner: click.testing.CliRunner) -> None:
    repo_1_url = str(data_dir / 'repos' / 'prj-1')
    repo_1_path = './prj-1'
    repo_2_url = str(data_dir / 'repos' / 'prj-2')
    repo_2_path = './prj-2'

    cli_runner.invoke(cli.main, ['init'])
    cli_runner.invoke(cli.main, ['add', repo_1_url, repo_1_path])
    cli_runner.invoke(cli.main, ['add', repo_2_url, repo_2_path])


def test_001_init_no_root_repo(
    data_dir: pathlib.Path, cli_runner: click.testing.CliRunner
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '001_wok.yml'
    )


def test_002_init_in_root_repo(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
) -> None:
    result = cli_runner.invoke(cli.main, ['init'])
    assert result.exit_code == 0, result.output

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '002_wok.yml'
    )


def test_003_init_in_root_repo_branch(
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


def test_011_add(data_dir: pathlib.Path, cli_runner: click.testing.CliRunner) -> None:
    cli_runner.invoke(cli.main, ['init'])
    repo_1_url = str(data_dir / 'repos' / 'prj-1')
    repo_1_path = './prj-1'
    repo_2_url = str(data_dir / 'repos' / 'prj-2')
    repo_2_path = './prj-2'

    result = cli_runner.invoke(cli.main, ['add', repo_1_url, repo_1_path])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))

    expected_config = config.Config.load(path=data_dir / '011_a_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=repo_1_path)
    assert repo_1.remotes['origin'].url == repo_1_url

    result = cli_runner.invoke(cli.main, ['add', repo_2_url, repo_2_path])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))

    expected_config = config.Config.load(path=data_dir / '011_b_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_2 = pygit2.Repository(path=repo_2_path)
    assert repo_2.remotes['origin'].url == repo_2_url


def test_021_start_no_root_repo(
    data_dir: pathlib.Path, cli_runner: click.testing.CliRunner, cooked: None
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '021_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config


def test_022_start_in_root_repo(
    data_dir: pathlib.Path,
    cli_runner: click.testing.CliRunner,
    root_repo: pygit2.Repository,
    cooked: None,
) -> None:
    result = cli_runner.invoke(cli.main, ['start', 'branch-1'])
    assert result.exit_code == 0, result.output

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '022_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    assert root_repo.head.shorthand == 'branch-1'
