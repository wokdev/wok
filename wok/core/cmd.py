import functools
import pathlib
import typing

import pygit2

from wok import config

from . import context


MaybeRepo_T = typing.Optional[pygit2.Repository]


def discover_root_repo() -> MaybeRepo_T:
    root_repo_path = pygit2.discover_repository(str(pathlib.Path.cwd()))
    if root_repo_path is None:
        return None
    return pygit2.Repository(path=root_repo_path)


def with_root_repo(func: typing.Callable) -> typing.Callable:
    @functools.wraps(func)
    def wrapper(*args: typing.Any, **kwargs: typing.Any) -> typing.Any:
        return func(*args, root_repo=discover_root_repo(), **kwargs)

    return wrapper


@context.with_conf_path
@with_root_repo
def init(conf_path: pathlib.Path, root_repo: MaybeRepo_T) -> None:
    conf = config.Config.create(path=conf_path)

    if root_repo is None:
        return

    conf.ref = root_repo.head.shorthand
    conf.save()


@context.with_conf
def add(conf: config.Config, path: pathlib.Path, url: str) -> None:
    # TODO: Assert path or url don't exist in the config already

    if path.exists():
        raise FileExistsError(path.absolute())

    repo = pygit2.clone_repository(url=url, path=str(path))

    ref = repo.head.shorthand

    repo_config = config.Repo(url=url, path=path, ref=ref)
    conf.repos.append(repo_config)
    conf.save()


@context.with_conf
@with_root_repo
def start(conf: config.Config, branch_name: str, root_repo: MaybeRepo_T) -> None:
    if root_repo is not None:
        try:
            ref = root_repo.lookup_reference_dwim(branch_name)
        except KeyError:
            pass
        else:
            raise ValueError(f"Reference `{ref.name}` already exists")

    conf.ref = branch_name
    conf.save()

    if root_repo is None:
        return

    root_repo.stash(stasher=root_repo.default_signature)
    started_branch = root_repo.branches.local.create(
        name=branch_name,
        commit=root_repo.resolve_refish(refish=root_repo.head.name)[0],
    )
    root_repo.checkout(refname=started_branch)
    root_repo.stash_pop()
