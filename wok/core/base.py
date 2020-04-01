import functools
import pathlib
import typing

import pygit2

from . import context


def is_clean(repo: pygit2.Repository) -> bool:
    return not [
        code
        for code in repo.status().values()
        if (code ^ (code & pygit2.GIT_STATUS_WT_NEW))
    ]


def require_clean(func: typing.Callable) -> typing.Callable:
    @functools.wraps(func)
    def wrapper(
        *args: typing.Any, ctx: context.Context, **kwargs: typing.Any
    ) -> typing.Any:

        if not is_clean(repo=ctx.root_repo):
            print(ctx.root_repo.status())
            raise ValueError(ctx.root_repo.workdir)

        for repo_conf in ctx.conf.repos:
            repo = pygit2.Repository(path=str(repo_conf.path))
            if not is_clean(repo=repo):
                raise ValueError(repo.workdir)

        return func(*args, ctx=ctx, **kwargs)

    return wrapper


def switch(repo: pygit2.Repository, ref: pygit2.Reference) -> None:
    stashed = False
    if not is_clean(repo=repo):
        repo.stash(stasher=repo.default_signature)
        stashed = True

    repo.checkout(refname=ref)

    if stashed:
        repo.stash_pop()


def commit(
    repo: pygit2.Repository,
    message: str,
    pathspecs: typing.Optional[typing.List[typing.Union[str, pathlib.Path]]] = None,
) -> None:
    if pathspecs is None:
        pathspecs = []

    pathspecs = [
        (
            str(
                pathspec.relative_to(repo.workdir)
                if pathspec.is_absolute()
                else pathspec
            )
            if isinstance(pathspec, pathlib.Path)
            else pathspec
        )
        for pathspec in pathspecs
    ]

    repo.index.add_all(pathspecs=pathspecs)
    repo.index.write()
    tree = repo.index.write_tree()

    try:
        parent, ref = repo.resolve_refish(refish=repo.head.name)
    except pygit2.GitError:
        parents = []
        ref_name = 'refs/heads/master'
    else:
        parents = [parent.oid]
        ref_name = ref.name

    repo.create_commit(
        ref_name, repo.default_signature, repo.default_signature, message, tree, parents
    )


def push(repo: pygit2.Repository, branch_name: str) -> None:
    branch = repo.branches.local[branch_name]
    if not branch.is_head():
        raise ValueError(branch)

    try:
        remote = repo.remotes['origin']
    except KeyError:
        return

    remote.push(specs=[branch.name])


def finish(repo: pygit2.Repository, branch_name: str, message: str) -> None:
    master = repo.branches.local['master']
    branch = repo.branches.local[branch_name]
    if not branch.is_head():
        raise ValueError(branch)

    merge_state, _ = repo.merge_analysis(branch.target, master.name)
    if merge_state & pygit2.GIT_MERGE_ANALYSIS_UP_TO_DATE:
        repo.checkout(refname=master)
        return
    if not (merge_state & pygit2.GIT_MERGE_ANALYSIS_FASTFORWARD):
        raise ValueError(branch)

    index: pygit2.Index = repo.merge_trees(ancestor=master, ours=master, theirs=branch)
    tree = index.write_tree(repo=repo)
    repo.create_commit(
        master.name,
        repo.default_signature,
        repo.default_signature,
        message,
        tree,
        [master.target],
    )
    repo.checkout(refname=master)
    branch.delete()


def tag(repo: pygit2.Repository, tag_name: str) -> None:
    repo.references.create(name=f'refs/tags/{tag_name}', target=repo.head.target)
