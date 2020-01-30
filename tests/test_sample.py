import os
import pathlib

import pytest

from wok import cli


@pytest.fixture(scope='session')
def sample_dir() -> pathlib.Path:
    return pathlib.Path(__file__).parent.joinpath('sample').absolute()


def test_001_sync(sample_dir: pathlib.Path) -> None:
    os.chdir(sample_dir)
    cli.sync()
    assert False
