import pathlib

import pygit2

from wok import config


def add(conf: config.Config, path: pathlib.Path, url: str) -> None:
    # TODO: Assert path or url don't exist in the config

    if path.exists():
        raise FileExistsError(path.absolute())

    repo = pygit2.clone_repository(url=url, path=str(path))

    ref = repo.head.shorthand

    repo_config = config.Repo(url=url, path=path, ref=ref)
    conf.repos.append(repo_config)
    conf.save()
