"""wok.toml loading, migration, and fixture-stable dumps."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from pathlib import Path

import tomli_w

from wok.errors import WokError

CONFIG_CURRENT_VERSION = "1.0"
_TOP_KEYS = {"version", "repo"}
_REPO_KEYS = {"path", "head", "skip_for"}


@dataclass
class Repo:
    """One configured repository."""

    path: Path
    head: str
    skip_for: list[str] = field(default_factory=list)

    def is_skipped_for(self, command: str) -> bool:
        return any(item.lower() == command.lower() for item in self.skip_for)


@dataclass
class Config:
    """Workspace configuration stored in wok.toml."""

    version: str = CONFIG_CURRENT_VERSION
    repos: list[Repo] = field(default_factory=list)

    @classmethod
    def new(cls) -> Config:
        return cls()

    def add_repo(self, path: Path, head: str) -> bool:
        if path.is_absolute():
            raise WokError(f"Repository path must be relative: `{path}`")
        if self._has_repo_path(path):
            return False
        self.repos.append(Repo(path=Path(path), head=head, skip_for=[]))
        return True

    def remove_repo(self, path: Path) -> bool:
        if path.is_absolute():
            raise WokError(f"Repository path must be relative: `{path}`")
        kept = [repo for repo in self.repos if repo.path != path]
        removed = len(kept) != len(self.repos)
        self.repos = kept
        return removed

    def require_known(self, paths: list[Path]) -> None:
        """Reject explicit paths that are not configured repositories."""
        known = {repo.path for repo in self.repos}
        for path in paths:
            if path not in known:
                raise WokError(f"Repository `{path}` is not in the workspace")

    def set_repo_head(self, path: Path, head: str) -> bool:
        if path.is_absolute():
            raise WokError(f"Repository path must be relative: `{path}`")
        updated = False
        for repo in self.repos:
            if repo.path == path and repo.head != head:
                repo.head = head
                updated = True
        return updated

    @classmethod
    def load(cls, config_path: Path) -> Config:
        try:
            text = cls.read(config_path)
            parsed = tomllib.loads(text)
        except WokError:
            raise
        except OSError as exc:
            raise WokError(f"Cannot read the wok file `{config_path}`") from exc
        except tomllib.TOMLDecodeError as exc:
            raise WokError(f"Cannot parse the wok file `{config_path}`") from exc

        if not isinstance(parsed, dict):
            raise WokError(f"Cannot parse the wok file `{config_path}`")
        _reject_unknown(parsed, _TOP_KEYS, "wok file")
        version = parsed.get("version")
        if not isinstance(version, str):
            raise WokError("Cannot parse the wok file: missing `version`")
        raw_repos = parsed.get("repo", [])
        if not isinstance(raw_repos, list):
            raise WokError("Cannot parse the wok file: `repo` must be an array")

        repos: list[Repo] = []
        for index, raw in enumerate(raw_repos):
            if not isinstance(raw, dict):
                raise WokError(
                    f"Cannot parse the wok file: repo {index} is not a table"
                )
            _reject_unknown(raw, _REPO_KEYS, f"repo {index}")
            path = raw.get("path")
            head = raw.get("head")
            skip_for = raw.get("skip_for", [])
            if not isinstance(path, str) or not isinstance(head, str):
                raise WokError(
                    f"Cannot parse the wok file: repo {index} needs path and head"
                )
            if not isinstance(skip_for, list) or not all(
                isinstance(item, str) for item in skip_for
            ):
                raise WokError(
                    f"Cannot parse the wok file: repo {index} skip_for must be strings"
                )
            repos.append(Repo(path=Path(path), head=head, skip_for=list(skip_for)))

        config = cls(version=version, repos=repos)
        if config.version == "1.0-experimental":
            config.version = CONFIG_CURRENT_VERSION
            config.save(config_path)
        return config

    @staticmethod
    def read(config_path: Path) -> str:
        try:
            return config_path.read_text()
        except OSError as exc:
            raise WokError(f"Cannot read the wok file `{config_path}`") from exc

    def save(self, config_path: Path) -> None:
        try:
            config_path.write_text(self.dump())
        except OSError as exc:
            raise WokError(f"Cannot save the wok file `{config_path}`") from exc

    def dump(self) -> str:
        """Serialize with tomli-w in the fixture layout."""
        if not self.repos:
            return tomli_w.dumps({"version": self.version, "repo": []})

        blocks = ["[[repo]]\n" + _repo_body(repo) for repo in self.repos]
        version_line = tomli_w.dumps({"version": self.version}).rstrip("\n")
        return version_line + "\n\n" + "\n\n".join(blocks) + "\n"

    def _has_repo_path(self, path: Path) -> bool:
        return any(repo.path == path for repo in self.repos)


def _repo_body(repo: Repo) -> str:
    lines = [
        f"path = {_toml_string(repo.path.as_posix())}",
        f"head = {_toml_string(repo.head)}",
    ]
    if repo.skip_for:
        items = ", ".join(_toml_string(item) for item in repo.skip_for)
        lines.append(f"skip_for = [{items}]")
    return "\n".join(lines)


def _toml_string(value: str) -> str:
    rendered = tomli_w.dumps({"k": value})
    return rendered.split("=", 1)[1].strip()


def _reject_unknown(data: dict[str, object], allowed: set[str], where: str) -> None:
    unknown = sorted(set(data) - allowed)
    if unknown:
        raise WokError(
            f"Cannot parse the wok file: unknown key `{unknown[0]}` in {where}"
        )
