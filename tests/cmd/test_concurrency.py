from io import StringIO

from wok.cmd import status
from wok.config import Config
from wok.runtime import Runtime, runtime_var


def test_parallel_status_keeps_config_order(repo_sample, monkeypatch) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    monkeypatch.setenv("WOK_JOBS", "4")
    runtime_var.set(Runtime(jobs=4, verbose=False))
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    text = output.getvalue()
    assert text.index("umbrella") < text.index("sub-a") < text.index("sub-b")


def test_jobs_one_status(repo_sample) -> None:
    from wok.runtime import Runtime, runtime_var

    sample = repo_sample(["sub-a"], "a.toml")
    runtime_var.set(Runtime(jobs=1, verbose=False))
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "✅ sub-a [main]: clean" in output.getvalue()
