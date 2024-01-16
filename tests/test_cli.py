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


def test_cli_add_folder():
    _ = subprocess.run(
        [
            "godata",
            "link",
            "-r",
            "cli_test",
            "data2",
            str(data_path),
        ]
    )
    p = load_project("cli_test")
    items = p.get("data2/test_ones.npy")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)
    subfolder_item = p.get("data2/more_data/more_test_ones.npy")
    assert np.all(subfolder_item == expected_data)


def test_cli_get():
    result = subprocess.run(
        [
            "godata",
            "get",
            "cli_test",
            "data/test_ones",
        ],
        capture_output=True,
    )
    output = result.stdout.decode("utf-8").strip()
    path = output.split("\n")[1]
    # check that the path returned is correct
    assert path == str(data_path / "test_ones.npy")


def test_cli_ie():
    result = subprocess.run(
        ["godata", "export", "cli_test", "-o", str(Path.cwd())],
        capture_output=True,
    )
    path = Path.cwd() / "cli_test.zip"
    assert path.exists()
    result = subprocess.run(
        [
            "godata",
            "import",
            str(path),
            "cli_test_import",
        ],
        capture_output=True,
    )

    p = load_project("cli_test_import")
    items = p.get("data/test_ones")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)
