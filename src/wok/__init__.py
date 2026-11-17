"""Git Wok -- control several git repositories as a single project."""

from importlib.metadata import PackageNotFoundError, version

DEFAULT_CONFIG_NAME = "wok.toml"

try:
    __version__ = version("wok")
except PackageNotFoundError:
    __version__ = "0.0.0"

__all__ = ["DEFAULT_CONFIG_NAME", "__version__"]
