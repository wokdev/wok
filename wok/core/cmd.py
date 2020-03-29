import pathlib

import pygit2

from wok import config

from . import context


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
    # TODO: Assert path or url don't exist in the config already

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

    stashed = False
    if not ctx.root_repo.status():
        ctx.root_repo.stash(stasher=ctx.root_repo.default_signature)
        stashed = True

    started_branch = ctx.root_repo.branches.local.create(
        name=branch_name,
        commit=ctx.root_repo.resolve_refish(refish=ctx.root_repo.head.name)[0],
    )
    ctx.root_repo.checkout(refname=started_branch)

    if stashed:
        ctx.root_repo.stash_pop()

    ctx.conf.ref = branch_name
    ctx.conf.save()
