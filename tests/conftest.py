import difflib
import os
import pathlib
import pprint
import sys
import typing

import attr
import click.testing
import pygit2
import pytest


def pytest_configure() -> None:
    sys.path.insert(0, str(pathlib.Path(__file__).parents[1]))


def pytest_assertrepr_compare(
    op: str, left: typing.Any, right: typing.Any
) -> typing.Optional[typing.Iterable[str]]:
    from wok import config

    if (
        op == '=='
        and isinstance(left, config.Config)
        and isinstance(right, config.Config)
    ):
        left = attr.assoc(left, _path='**excluded**')
        right = attr.assoc(right, _path='**excluded**')
        return [
            "Comparing Config instances:",
            *difflib.ndiff(
                pprint.pformat(attr.asdict(left), width=1, compact=False).split('\n'),
                pprint.pformat(attr.asdict(right), width=1, compact=False).split('\n'),
            ),
        ]

    return None


@pytest.fixture(scope='session')
def tests_dir() -> pathlib.Path:
    return pathlib.Path(__file__).parent.absolute()


@pytest.fixture(scope='session')
def data_dir(tests_dir: pathlib.Path) -> pathlib.Path:
    return tests_dir / 'data'


@pytest.fixture(scope='session')
def repo_1_url(data_dir: pathlib.Path) -> str:
    return str(data_dir / 'repos' / 'prj-1')


@pytest.fixture(scope='session')
def repo_1_path() -> pathlib.Path:
    return pathlib.Path('./prj-1')


@pytest.fixture(scope='session')
def repo_2_url(data_dir: pathlib.Path) -> str:
    return str(data_dir / 'repos' / 'prj-2')


@pytest.fixture(scope='session')
def repo_2_path() -> pathlib.Path:
    return pathlib.Path('./prj-2')


@pytest.fixture()
def tmp_cwd(tmp_path: pathlib.Path) -> typing.Iterator[pathlib.Path]:
    orig_cwd = pathlib.Path.cwd()
    os.chdir(path=tmp_path)
    yield tmp_path
    os.chdir(orig_cwd)


@pytest.fixture()
def root_repo(tmp_cwd: pathlib.Path) -> pygit2.Repository:
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
    root_repo: pygit2.Repository,
    repo_1_url: str,
    repo_1_path: pathlib.Path,
    repo_2_url: str,
    repo_2_path: pathlib.Path,
) -> pygit2.Repository:
    from wok import cli

    cli_runner = click.testing.CliRunner()
    assert (
        cli_runner.invoke(cli.main, ['init']).exit_code
        | cli_runner.invoke(cli.main, ['add', repo_1_url, str(repo_1_path)]).exit_code
        | cli_runner.invoke(cli.main, ['add', repo_2_url, str(repo_2_path)]).exit_code
        | cli_runner.invoke(cli.main, ['commit']).exit_code
    ) == 0
    return root_repo
