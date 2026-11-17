"""Wok command implementations."""

from wok.cmd.assemble import assemble
from wok.cmd.auth import check_auth as test_auth
from wok.cmd.completion import completion
from wok.cmd.init import init
from wok.cmd.lock import lock
from wok.cmd.push import push
from wok.cmd.status import status
from wok.cmd.switch import switch
from wok.cmd.tag import tag, tag_create, tag_list, tag_push
from wok.cmd.update import update

__all__ = [
    "assemble",
    "completion",
    "init",
    "lock",
    "push",
    "status",
    "switch",
    "tag",
    "tag_create",
    "tag_list",
    "tag_push",
    "test_auth",
    "update",
]
