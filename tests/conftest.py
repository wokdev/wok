import difflib
import os
import pathlib
import pprint
import shutil
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


@pytest.fixture(scope='session')
def repo_w_url(data_dir: pathlib.Path) -> str:
    return str(data_dir / 'repos' / 'workspace')


@pytest.fixture()
def tmp_cwd(tmp_path: pathlib.Path) -> typing.Iterator[pathlib.Path]:
    orig_cwd = pathlib.Path.cwd()
    os.chdir(path=tmp_path)
    yield tmp_path
    os.chdir(orig_cwd)


@pytest.fixture()
def empty_repo(tmp_cwd: pathlib.Path) -> pygit2.Repository:
    return pygit2.init_repository(path=str(pathlib.Path.cwd()))


@pytest.fixture()
def root_repo(tmp_cwd: pathlib.Path) -> pygit2.Repository:
    import wok.core.base

    cwd = pathlib.Path.cwd()

    root_repo = pygit2.init_repository(path=str(cwd))

    cwd.joinpath('readme').write_text('a file to commit')

    wok.core.base.commit(repo=root_repo, message='Initial commit')

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


@pytest.fixture()
def tmp_repos(
    repo_1_url: str,
    repo_1_path: pathlib.Path,
    repo_2_url: str,
    repo_2_path: pathlib.Path,
    cooked_repo: pygit2.Repository,
    repo_w_url: str,
) -> typing.Iterator[typing.Iterable[pygit2.Repository]]:
    repos = (
        pygit2.Repository(path=str(repo_1_path)),
        pygit2.Repository(path=str(repo_2_path)),
        cooked_repo,
    )
    repo_urls = (repo_1_url, repo_2_url, repo_w_url)

    for n, repo in enumerate(repos):
        repo_url = repo_urls[n]
        repo_tmp_url = f'{repo_url}-tmp'
        shutil.copytree(src=repo_url, dst=repo_tmp_url)
        repo.remotes.set_url(name='origin', url=repo_tmp_url)

    yield repos

    for n, repo in enumerate(repos):
        repo_tmp_url = repo.remotes['origin'].url
        repo.remotes.set_url(name='origin', url=repo_urls[n])
        shutil.rmtree(path=repo_tmp_url)
