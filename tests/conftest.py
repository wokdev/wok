import pathlib
import sys


def pytest_configure():
    sys.path.insert(0, str(pathlib.Path(__file__).parents[1]))
