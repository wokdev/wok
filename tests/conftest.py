import difflib
import pathlib
import pprint
import sys
import typing

import attr
import pytest


def pytest_configure() -> None:
    sys.path.insert(0, str(pathlib.Path(__file__).parents[1]))


def pytest_assertrepr_compare(
    op: str, left: typing.Any, right: typing.Any
) -> typing.Optional[typing.Iterable[str]]:
    from wok import config

    if (
        op == '=='
        and isinstance(left, config.Config)
        and isinstance(right, config.Config)
    ):
        left = attr.assoc(left, _path='**excluded**')
        right = attr.assoc(right, _path='**excluded**')
        return [
            "Comparing Config instances:",
            *difflib.ndiff(
                pprint.pformat(attr.asdict(left), width=1, compact=False).split('\n'),
                pprint.pformat(attr.asdict(right), width=1, compact=False).split('\n'),
            ),
        ]

    return None


@pytest.fixture(scope='session')
def tests_dir() -> pathlib.Path:
    return pathlib.Path(__file__).parent.absolute()


@pytest.fixture(scope='session')
def data_dir(tests_dir: pathlib.Path) -> pathlib.Path:
    return tests_dir / 'data'
