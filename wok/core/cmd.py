import pathlib

import pygit2

from wok import config

from . import context


@context.with_conf_path
def init(conf_path: pathlib.Path) -> None:
    conf = config.Config.create(path=conf_path)

    root_repo_path = pygit2.discover_repository(str(pathlib.Path.cwd()))
    if root_repo_path is None:
        return

    root_repo = pygit2.Repository(path=root_repo_path)
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
