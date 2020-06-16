import functools
import pathlib
import typing

import click
import pygit2

from . import context


def is_clean(repo: pygit2.Repository) -> bool:
    return not any(
        code
        for code in repo.status().values()
        if (
            code
            ^ (code & pygit2.GIT_STATUS_WT_NEW)
            ^ (code & pygit2.GIT_STATUS_IGNORED)
        )
    )


def require_clean(func: typing.Callable) -> typing.Callable:
    @functools.wraps(func)
    def wrapper(
        *args: typing.Any, ctx: context.Context, **kwargs: typing.Any
    ) -> typing.Any:

        if not is_clean(repo=ctx.root_repo):
            print(ctx.root_repo.status())
            raise ValueError(ctx.root_repo.workdir)

        for repo_conf in ctx.conf.repos:
            repo = pygit2.Repository(str(repo_conf.path))
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

    remote.push(specs=[f'{branch.name}'], callbacks=RemoteCallbacks())
    upstream_branch = repo.branches.get(f'{remote.name}/{branch.shorthand}')
    if upstream_branch is not None:
        branch.upstream = upstream_branch


def sync(repo: pygit2.Repository, branch_name: str) -> None:
    """
    Tries to update the `branch_name` branch of the `repo` repo to the latest
    upstream branch state.
    If the branch is up to date, does nothing.
    If the branch can be fast-forwarded, resets to the upstream.
    Otherwise, fails with an error.
    """
    branch = repo.branches.local[branch_name]
    if not branch.is_head():
        raise ValueError(branch)

    try:
        remote = repo.remotes['origin']
    except KeyError:
        return

    remote.fetch(callbacks=RemoteCallbacks())
    upstream_branch = branch.upstream
    if not upstream_branch:
        return

    merge_state, _ = repo.merge_analysis(upstream_branch.target, branch.name)
    if merge_state & pygit2.GIT_MERGE_ANALYSIS_UP_TO_DATE:
        return
    if not (merge_state & pygit2.GIT_MERGE_ANALYSIS_FASTFORWARD):
        raise ValueError(branch)

    repo.reset(upstream_branch.target, pygit2.GIT_RESET_HARD)
    repo.checkout(refname=branch)


def finish(repo: pygit2.Repository, branch_name: str, message: str) -> None:
    master = repo.branches.local['master']
    branch = repo.branches.local[branch_name]
    if not branch.is_head():
        raise ValueError(branch)

    merge_squash(repo=repo, ours_branch=master, theirs_branch=branch, message=message)

    repo.checkout(refname=master)
    branch.delete()


def merge_squash(
    repo: pygit2.Repository,
    ours_branch: pygit2.Branch,
    theirs_branch: pygit2.Branch,
    message: str,
) -> None:
    """
    Performs a merge of the `theirs_branch` into `ours_branch` sqaushing the commits
    """
    merge_state, _ = repo.merge_analysis(theirs_branch.target, ours_branch.name)
    if merge_state & pygit2.GIT_MERGE_ANALYSIS_UP_TO_DATE:
        return
    if not (merge_state & pygit2.GIT_MERGE_ANALYSIS_FASTFORWARD):
        raise ValueError(theirs_branch)

    index: pygit2.Index = repo.merge_trees(
        ancestor=ours_branch, ours=ours_branch, theirs=theirs_branch
    )
    tree = index.write_tree(repo=repo)
    repo.create_commit(
        ours_branch.name,
        repo.default_signature,
        repo.default_signature,
        message,
        tree,
        [ours_branch.target],
    )


def tag(repo: pygit2.Repository, tag_name: str) -> None:
    repo.references.create(name=f'refs/tags/{tag_name}', target=repo.head.target)


def clone(url: str, path: pathlib.Path, **kwargs: typing.Any) -> pygit2.Repository:
    return pygit2.clone_repository(
        url=url, path=str(path), callbacks=RemoteCallbacks(), **kwargs
    )


class RemoteCallbacks(pygit2.RemoteCallbacks):
    """
    Overrides the credentials callback.
    """

    def credentials(
        self, url: str, username_from_url: typing.Optional[str], allowed_types: int
    ) -> typing.Optional[
        typing.Union[pygit2.Keypair, pygit2.UserPass, pygit2.Username]
    ]:
        """
        If the remote server requires authentication, this function will be called and
        its return value used for authentication.
        """
        if allowed_types & pygit2.credentials.GIT_CREDTYPE_SSH_KEY:
            return pygit2.KeypairFromAgent(username_from_url)
        elif allowed_types & pygit2.credentials.GIT_CREDTYPE_USERPASS_PLAINTEXT:
            username = username_from_url or click.prompt("Username")
            password = click.prompt("Password", hide_input=True)
            return pygit2.UserPass(username=username, password=password)
        elif allowed_types & pygit2.credentials.GIT_CREDTYPE_USERNAME:
            return pygit2.Username(username_from_url)
        else:
            return None
