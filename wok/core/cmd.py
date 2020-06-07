import pathlib
import sys
import typing

import pygit2

from wok import config

from . import base, context


@context.with_context
def init(*, ctx: context.Context) -> None:
    conf = config.Config.create(path=ctx.conf_path)
    if not ctx.root_repo.head_is_unborn:
        conf.ref = ctx.root_repo.head.shorthand
    conf.save()
    commit(message="Add `wok` config")


@context.with_context
def commit(*, ctx: context.Context, message: str = "Update `wok` config") -> None:
    base.commit(repo=ctx.root_repo, message=message, pathspecs=[ctx.conf_path])


@context.with_context
def add(*, ctx: context.Context, path: pathlib.Path, url: str) -> None:
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

    repo = base.clone(url=url, path=path)

    ref = repo.head.shorthand

    repo_config = config.Repo(url=url, path=path, ref=ref)
    ctx.conf.repos.append(repo_config)
    ctx.conf.save()


@context.with_context
def start(*, ctx: context.Context, branch_name: str) -> None:
    if ctx.root_repo.head_is_unborn:
        raise ValueError("The workspace is not initialized. Run `wok init`.")
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
def join(*, ctx: context.Context, repo_paths: typing.Iterable[pathlib.Path]) -> None:
    repo_confs: typing.MutableSequence[config.Repo] = []

    for repo_path in repo_paths:
        repo_conf = ctx.conf.lookup_repo(path=str(repo_path))
        if repo_conf is None:
            print(f"Unknown repo path `{repo_path}`")
            raise ValueError(repo_path)

        repo_confs.append(repo_conf)

    branch_name = ctx.conf.ref

    for repo_conf in repo_confs:
        repo = pygit2.Repository(str(repo_conf.path))

        try:
            ref = repo.lookup_reference_dwim(branch_name)
        except KeyError:
            ref = repo.branches.local.create(
                name=branch_name, commit=repo.resolve_refish(refish=repo.head.name)[0]
            )

        base.switch(repo=repo, ref=ref)

        repo_conf.ref = ref.shorthand
        ctx.conf.save()


@context.with_context
def push(*, ctx: context.Context) -> None:
    for repo_conf in ctx.conf.joined_repos:
        repo = pygit2.Repository(str(repo_conf.path))
        base.push(repo=repo, branch_name=repo_conf.ref)

    base.push(repo=ctx.root_repo, branch_name=ctx.conf.ref)


@context.with_context
def finish(*, ctx: context.Context, message: str) -> None:
    # TODO: Implement different merge strategies
    # TODO: Allow to set integration branch different from `master`
    # TODO: Use GIT_EDITOR to compose the finish message

    if ctx.conf.ref == 'master':
        raise ValueError('master')

    for repo_conf in ctx.conf.joined_repos:
        repo = pygit2.Repository(str(repo_conf.path))
        base.finish(repo=repo, branch_name=repo_conf.ref, message=message)
        repo_conf.ref = 'master'

    branch_name = ctx.conf.ref
    ctx.conf.ref = 'master'
    ctx.conf.save()
    commit()
    base.finish(repo=ctx.root_repo, branch_name=branch_name, message=message)


@context.with_context
@base.require_clean
def tag(*, ctx: context.Context, tag_name: str) -> None:
    base.tag(repo=ctx.root_repo, tag_name=tag_name)

    for repo_conf in ctx.conf.repos:
        repo = pygit2.Repository(str(repo_conf.path))
        base.tag(repo=repo, tag_name=tag_name)


@context.with_context
@base.require_clean
def sync(*, ctx: context.Context) -> None:
    base.switch(
        repo=ctx.root_repo, ref=ctx.root_repo.lookup_reference_dwim(ctx.conf.ref)
    )
    base.sync(repo=ctx.root_repo, branch_name=ctx.conf.ref)

    for repo_conf in ctx.conf.repos:
        repo = pygit2.Repository(str(repo_conf.path))
        base.switch(repo=repo, ref=repo.lookup_reference_dwim(repo_conf.ref))
        base.sync(repo=repo, branch_name=repo_conf.ref)


@context.with_context
@base.require_clean
def fork(*, ctx: context.Context, branch_name: str) -> None:
    try:
        ref = ctx.root_repo.lookup_reference_dwim(branch_name)
    except KeyError:
        pass
    else:
        raise ValueError(f"Reference `{ref.name}` already exists")

    forked_branch = ctx.root_repo.branches.local.create(
        name=branch_name,
        commit=ctx.root_repo.resolve_refish(refish=ctx.root_repo.head.name)[0],
    )
    base.switch(repo=ctx.root_repo, ref=forked_branch)
    ctx.conf.ref = branch_name
    ctx.conf.save()

    for repo_conf in ctx.conf.repos:
        repo = pygit2.Repository(str(repo_conf.path))

        try:
            ref = repo.lookup_reference_dwim(branch_name)
        except KeyError:
            ref = repo.branches.local.create(
                name=branch_name, commit=repo.resolve_refish(refish=repo.head.name)[0]
            )
        else:
            raise ValueError(
                f"Reference `{ref.name}` already exists in repo `{repo_conf.path}`"
            )

        base.switch(repo=repo, ref=ref)

        repo_conf.ref = ref.shorthand
        ctx.conf.save()
