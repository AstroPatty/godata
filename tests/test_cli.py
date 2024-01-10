import subprocess
from pathlib import Path


def test_cli_create():
    _ = subprocess.run(["godata", "project", "create", "cli_test"])
    expected_location = Path.cwd() / "cli_test"
    assert expected_location.exists() and expected_location.is_dir()
