import os
import subprocess
from pathlib import Path

import numpy as np

from godata import load_project

data_path = Path(os.environ.get("DATA_PATH"))


def test_cli_create():
    _ = subprocess.run(["godata", "create", "cli_test"])
    expected_location = Path.cwd() / "default.cli_test"
    assert expected_location.exists() and expected_location.is_dir()


def test_cli_create_in_collection():
    _ = subprocess.run(["godata", "create", "cli_test_collection/cli_test"])
    expected_location = Path.cwd() / "cli_test_collection.cli_test"
    assert expected_location.exists() and expected_location.is_dir()


def test_cli_link_file():
    _ = subprocess.run(
        [
            "godata",
            "link",
            "cli_test",
            "data/test_ones",
            str(data_path / "test_ones.npy"),
        ]
    )
    p = load_project("cli_test")
    items = p.get("data/test_ones")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)
