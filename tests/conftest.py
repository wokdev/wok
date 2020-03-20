import pathlib
import sys

import pytest


def pytest_configure():
    sys.path.insert(0, str(pathlib.Path(__file__).parents[1]))


@pytest.fixture(scope='session')
def tests_dir() -> pathlib.Path:
    return pathlib.Path(__file__).parent.absolute()


@pytest.fixture(scope='session')
def data_dir(tests_dir: pathlib.Path) -> pathlib.Path:
    return tests_dir / 'data'
