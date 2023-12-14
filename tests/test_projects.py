import os
from pathlib import Path

import numpy as np
import pandas as pd
import pytest

from godata import create_project, list_collections, list_projects
from godata.project import GodataProjectError

data_path = Path(os.environ.get("DATA_PATH"))


def test_create():
    p = create_project("test1")
    items = p.ls()
    assert p.name == "test1" and p.collection == "default" and not items


def test_create_duplicate():
    p2 = create_project("test2")
    with pytest.raises(GodataProjectError):
        p2 = create_project("test2")


def test_add_file():
    p = create_project("test3")
    p.link(data_path / "test_ones.npy", "data/test_data")
    items = p.get("data/test_data")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)


def test_add_folder():
    p = create_project("test4")
    p.link(data_path, "data")
    items = p.get("data/test_ones.npy")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)


def test_store_file():
    p = create_project("test5")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    items = p.get("data/test_data")
    assert np.all(items == expected_data)


def test_overwrite():
    p = create_project("test6")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    stored_path = p.get("data/test_data", as_path=True)

    df_data = pd.read_csv(data_path / "test_df.csv")
    p.store(df_data, "data/test_data")
    data = p.get("data/test_data")
    assert np.all(data.values == df_data.values)
    assert not stored_path.exists()


def test_exists():
    p = create_project("test7")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    hp1 = p.has_path("data/test_data")
    hp2 = p.has_path("data/test_data2")
    # Check the json parsing as well
    assert type(hp1) == bool and type(hp2) == bool
    assert hp1 and not hp2


def test_delete_file():
    p = create_project("test8")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    p.store(expected_data, "data2/test_data")
    p.remove("data/test_data")
    hp1 = p.has_path("data/test_data")
    children = p.list()
    assert not hp1 and not children["files"] and children["folders"] == ["data2"]


def test_list_collections():
    p = create_project("test9", "test_collection")
    collections = list_collections()
    assert collections == ["default", "test_collection"]


def test_list_projects():
    p = create_project("test10", "test_collection")
    projects = list_projects("test_collection")
    assert "test10" in projects


def test_project_path_clean():
    p = create_project("test11")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "/data/test_data/")
    data = p.get("/data/test_data")
    data2 = p.get("data/test_data/")
    assert np.all(data == data2)
    assert np.all(data == expected_data)
