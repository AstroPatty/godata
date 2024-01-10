import subprocess
from pathlib import Path


def test_cli_create():
    _ = subprocess.run(["godata", "create", "cli_test"])
    expected_location = Path.cwd() / "default.cli_test"
    assert expected_location.exists() and expected_location.is_dir()


def test_cli_create_in_collection():
    _ = subprocess.run(["godata", "create", "cli_test_collection/cli_test"])
    expected_location = Path.cwd() / "cli_test_collection.cli_test"
    assert expected_location.exists() and expected_location.is_dir()
