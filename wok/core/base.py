import pygit2


def switch(repo: pygit2.Repository, ref: pygit2.Reference) -> None:
    stashed = False
    if [
        code
        for code in repo.status().values()
        if (code ^ (code & pygit2.GIT_STATUS_WT_NEW))
    ]:
        repo.stash(stasher=repo.default_signature)
        stashed = True

    repo.checkout(refname=ref)

    if stashed:
        repo.stash_pop()
