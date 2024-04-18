import os
import shutil
import time
from pathlib import Path

import numpy as np
import pandas as pd
import polars as pl
import pytest

from godata import create_project, list_collections, list_projects, load_project
from godata.ie import export_project, import_project
from godata.project import GodataProjectError

data_path = Path(os.environ.get("DATA_PATH"))


def test_create():
    p = create_project("test1")
    items = p.ls()
    assert p.name == "test1" and p.collection == "default" and not items


def test_create_duplicate():
    _ = create_project("test2")
    with pytest.raises(GodataProjectError):
        _ = create_project("test2")


def test_list_collections():
    _ = create_project("test9", "test_collection")
    collections = list_collections()
    assert collections.sort() == ["default", "test_collection"].sort()


def test_list_projects():
    _ = create_project("test10", "test_collection")
    projects = list_projects("test_collection")
    assert "test10" in projects


def test_load_project():
    p = create_project("test12")
    expected_data = np.random.rand(10, 10)
    p.store(expected_data, "data/test_data")
    del p
    p = load_project("test12")
    data = p.get("data/test_data")
    assert np.all(data == expected_data)


def test_ie():
    p = create_project("test13")
    expected_data = np.random.rand(10, 10)
    p.store(expected_data, "data/test_data")
    p.link(data_path, "data2", recursive=True)
    output_path = export_project("test13")
    assert output_path.exists()
    import_project(output_path, "test_import", verbose=True)
    p2 = load_project("test_import")
    # get the list of folders in this path
    data = p2.get("data/test_data")
    assert np.all(data == expected_data)
