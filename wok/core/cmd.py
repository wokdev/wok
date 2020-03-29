import pathlib
import sys
import typing

import pygit2

from wok import config

from . import base, context


@context.with_context
def init(ctx: context.Context) -> None:
    conf = config.Config.create(path=ctx.conf_path)
    conf.ref = ctx.root_repo.head.shorthand
    conf.save()


@context.with_context
def commit(ctx: context.Context) -> None:
    ctx.root_repo.index.add(str(ctx.conf_path))
    ctx.root_repo.index.write()
    tree = ctx.root_repo.index.write_tree()
    parent, ref = ctx.root_repo.resolve_refish(refish=ctx.root_repo.head.name)
    ctx.root_repo.create_commit(
        ref.name,
        ctx.root_repo.default_signature,
        ctx.root_repo.default_signature,
        "Update `wok` config",
        tree,
        [parent.oid],
    )


@context.with_context
def add(ctx: context.Context, path: pathlib.Path, url: str) -> None:
    already_configured = ctx.conf.lookup_repo(url=url, path=path)
    if already_configured is not None:
        print(
            f"Repo `{already_configured.url}` is already congigured at path"
            f" `{already_configured.path}`",
            file=sys.stderr,
        )
        raise ValueError(already_configured)
    if path.exists():
        raise FileExistsError(path.absolute())

    repo = pygit2.clone_repository(url=url, path=str(path))

    ref = repo.head.shorthand

    repo_config = config.Repo(url=url, path=path, ref=ref)
    ctx.conf.repos.append(repo_config)
    ctx.conf.save()


@context.with_context
def start(ctx: context.Context, branch_name: str) -> None:
    try:
        ref = ctx.root_repo.lookup_reference_dwim(branch_name)
    except KeyError:
        pass
    else:
        raise ValueError(f"Reference `{ref.name}` already exists")

    started_branch = ctx.root_repo.branches.local.create(
        name=branch_name,
        commit=ctx.root_repo.resolve_refish(refish=ctx.root_repo.head.name)[0],
    )
    base.switch(repo=ctx.root_repo, ref=started_branch)
    ctx.conf.ref = branch_name
    ctx.conf.save()


@context.with_context
def join(ctx: context.Context, repo_paths: typing.Iterable[str]) -> None:
    repo_confs: typing.MutableSequence[config.Repo] = []

    for repo_path in repo_paths:
        repo_conf = ctx.conf.lookup_repo(path=repo_path)
        if repo_conf is None:
            print(f"Unknown repo path `{repo_path}`")
            raise ValueError(repo_path)

        repo_confs.append(repo_conf)

    branch_name = ctx.conf.ref

    for repo_conf in repo_confs:
        repo = pygit2.Repository(path=str(repo_conf.path))

        try:
            ref = repo.lookup_reference_dwim(branch_name)
        except KeyError:
            ref = repo.branches.local.create(
                name=branch_name, commit=repo.resolve_refish(refish=repo.head.name)[0]
            )

        base.switch(repo=repo, ref=ref)

        repo_conf.ref = ref.shorthand
        ctx.conf.save()
