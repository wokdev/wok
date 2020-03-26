import pathlib
import sys
import typing

import pytest


def pytest_configure() -> None:
    sys.path.insert(0, str(pathlib.Path(__file__).parents[1]))


def pytest_assertrepr_compare(
    op: str,
    left: typing.Union[pathlib.Path, typing.Any],
    right: typing.Union[pathlib.Path, typing.Any],
) -> typing.Optional[typing.Iterable[str]]:
    if (
        op == "=="
        and isinstance(left, pathlib.Path)
        and isinstance(right, pathlib.Path)
        and left.suffix in ('.yml', '.yaml')
        and right.suffix in ('.yml', '.yaml')
    ):
        
        return [
            "Comparing Foo instances:",
            "   vals: {} != {}".format(left.val, right.val),
        ]


@pytest.fixture(scope='session')
def tests_dir() -> pathlib.Path:
    return pathlib.Path(__file__).parent.absolute()


@pytest.fixture(scope='session')
def data_dir(tests_dir: pathlib.Path) -> pathlib.Path:
    return tests_dir / 'data'
