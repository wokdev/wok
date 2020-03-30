import pathlib
import typing

import _pytest
import pygit2
import pytest

import wok.core.base
from wok import config, core


@pytest.fixture(autouse=True)
def ctx(monkeypatch: _pytest.monkeypatch.MonkeyPatch) -> None:
    def _get_current_context() -> object:
        class FakeContext:
            obj: core.Context = core.Context()

        return FakeContext()

    monkeypatch.setattr('click.get_current_context', _get_current_context)


def test_000_context_fails_without_root_repo(tmp_cwd: pathlib.Path) -> None:
    with pytest.raises(FileNotFoundError):
        core.Context()


def test_002_init(data_dir: pathlib.Path, root_repo: pygit2.Repository) -> None:
    core.init()

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '002_wok.yml'
    )


def test_003_init_in_branch(
    data_dir: pathlib.Path, root_repo: pygit2.Repository
) -> None:
    dev_branch = root_repo.branches.local.create(
        name='dev', commit=next(root_repo.walk(root_repo.head.target))
    )
    root_repo.checkout(refname=dev_branch, strategy=pygit2.GIT_CHECKOUT_FORCE)

    core.init()

    assert config.Config.load(path=pathlib.Path('wok.yml')) == config.Config.load(
        path=data_dir / '003_wok.yml'
    )


def test_011_commit(root_repo: pygit2.Repository) -> None:
    core.init()
    core.commit()

    assert root_repo.status() == {}


def test_021_add(
    data_dir: pathlib.Path,
    root_repo: pygit2.Repository,
    repo_1_url: str,
    repo_1_path: pathlib.Path,
    repo_2_url: str,
    repo_2_path: pathlib.Path,
) -> None:
    core.init()
    core.add(path=repo_1_path, url=repo_1_url)

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '021_a_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=str(repo_1_path))
    assert repo_1.remotes['origin'].url == repo_1_url

    core.add(path=repo_2_path, url=repo_2_url)

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '021_b_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_2 = pygit2.Repository(path=str(repo_2_path))
    assert repo_2.remotes['origin'].url == repo_2_url


def test_022_add_fails_on_already_added(
    data_dir: pathlib.Path,
    cooked_repo: pygit2.Repository,
    repo_1_url: str,
    repo_1_path: pathlib.Path,
) -> None:
    with pytest.raises(ValueError):
        core.add(path=repo_1_path, url=repo_1_url)

    with pytest.raises(ValueError):
        core.add(path='other_path', url=repo_1_url)

    with pytest.raises(ValueError):
        core.add(path=repo_1_path, url='other_url')


def test_031_start(data_dir: pathlib.Path, cooked_repo: pygit2.Repository) -> None:
    core.start(branch_name='branch-1')

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '031_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    assert cooked_repo.head.shorthand == 'branch-1'


def test_041_join(
    data_dir: pathlib.Path, cooked_repo: pygit2.Repository, repo_1_path: pathlib.Path
) -> None:
    core.start(branch_name='branch-1')
    core.join(repo_paths=[repo_1_path])

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '041_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=str(repo_1_path))
    assert repo_1.head.shorthand == 'branch-1'


def test_042_join_many(
    data_dir: pathlib.Path,
    cooked_repo: pygit2.Repository,
    repo_1_path: pathlib.Path,
    repo_2_path: pathlib.Path,
) -> None:
    core.start(branch_name='branch-1')
    core.join(repo_paths=[repo_1_path, repo_2_path])

    actual_config = config.Config.load(path=pathlib.Path('wok.yml'))
    expected_config = config.Config.load(path=data_dir / '042_wok.yml')
    expected_config.repos[0].url = str(data_dir / expected_config.repos[0].url)
    expected_config.repos[1].url = str(data_dir / expected_config.repos[1].url)
    assert actual_config == expected_config

    repo_1 = pygit2.Repository(path=str(repo_1_path))
    repo_2 = pygit2.Repository(path=str(repo_2_path))
    assert repo_1.head.shorthand == 'branch-1'
    assert repo_2.head.shorthand == 'branch-1'


def test_043_join_fails_on_unknown_path(
    data_dir: pathlib.Path, cooked_repo: pygit2.Repository
) -> None:
    core.start(branch_name='branch-1')

    with pytest.raises(ValueError):
        core.join(repo_paths=['unknown/path'])


def test_051_push(
    data_dir: pathlib.Path,
    repo_1_path: pathlib.Path,
    repo_2_path: pathlib.Path,
    tmp_repos: typing.Iterable[pygit2.Repository],
) -> None:
    repo_1, repo_2, cooked_repo = tmp_repos
    repo_w_tmp_url = cooked_repo.remotes['origin'].url
    cooked_repo.remotes.delete('origin')

    core.start(branch_name='branch-1')
    core.join(repo_paths=[repo_1_path])
    core.commit()

    repo_1_change = repo_1_path.joinpath('change-1')
    repo_1_change.write_text('added changes 1')
    wok.core.base.commit(repo=repo_1, message="Some changes", pathspecs=[repo_1_change])

    repo_2_change = repo_2_path.joinpath('change-2')
    repo_2_change.write_text('added changes 2')
    wok.core.base.commit(repo=repo_2, message="Some changes", pathspecs=[repo_2_change])

    core.push()

    assert 'origin/branch-1' not in cooked_repo.branches.remote
    assert 'origin/branch-1' in repo_1.branches.remote
    assert 'origin/branch-1' not in repo_2.branches.remote

    cooked_repo.create_remote(name='origin', url=repo_w_tmp_url)

    core.push()

    assert 'origin/branch-1' in cooked_repo.branches.remote
