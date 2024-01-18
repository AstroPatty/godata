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


def setup_module(module):
    # Make sure the server is running
    from godata.server import start

    start()


def teardown_module(module):
    # Make sure the server is stopped
    from godata.server import stop

    stop()


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


def test_add_with_metadata():
    p = load_project("test3")
    p.link(data_path / "test_ones.npy", "data/test_meta", metadata={"test": "test"})
    items = p.get("data/test_meta")
    expected_data = np.ones((10, 10))
    expected_metadata = {"test": "test"}
    path = p.get("data/test_meta", as_path=True)
    expected_metadata["real_path"] = str(path)
    assert np.all(items == expected_data)
    found_metadata = p.get_metadata("data/test_meta")
    assert found_metadata == expected_metadata


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
    metadata = p.get_metadata("data/test_data")
    assert metadata["obj_type"] == "numpy.ndarray"


def test_overwrite():
    p = create_project("test6")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    stored_path = p.get("data/test_data", as_path=True)

    df_data = pd.read_csv(data_path / "test_df.csv")
    with pytest.raises(GodataProjectError):
        p.store(df_data, "data/test_data")
    p.store(df_data, "data/test_data", overwrite=True)
    data = p.get("data/test_data")
    assert np.all(data.values == df_data.values)
    assert not stored_path.exists()


def test_store_different_type():
    p = load_project("test6")
    df_data = pd.read_csv(data_path / "test_df.csv")
    p.store(df_data, "data/test_data_parquet", format=".parquet")
    stored_path = p.get("data/test_data_parquet", as_path=True)
    assert stored_path.suffix == ".parquet"
    read_df = pd.read_parquet(stored_path)
    assert np.all(read_df.values == df_data.values)
    read_df = p.get("data/test_data_parquet")
    assert np.all(read_df.values == df_data.values)


def test_get_different_type():
    p = load_project("test6")
    stored_data = p.get("data/test_data", load_type=pl.DataFrame)
    assert type(stored_data) == pl.DataFrame
    stored_data = p.get("data/test_data")
    assert type(stored_data) == pd.DataFrame


def test_store_with_kwargs():
    p = load_project("test6")
    df_data = pd.read_csv(data_path / "test_df.csv")
    p.store(df_data, "data/test_data_windex", writer_kwargs={"index": True})
    stored_path = p.get("data/test_data_windex", as_path=True)
    read_df = pd.read_csv(stored_path)
    assert "Unnamed: 0" in read_df.columns and not "Unnamed: 0" in df_data.columns
    # This function will fail if defaults for pandas change


def test_read_with_kwargs():
    p = load_project("test6")
    data = p.get("data/test_data_windex", reader_kwargs={"nrows": 2})
    assert len(data) == 2


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
    assert collections.sort() == ["default", "test_collection"].sort()


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


def test_load_project():
    p = create_project("test12")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    del p
    p = load_project("test12")
    data = p.get("data/test_data")
    assert np.all(data == expected_data)


def test_ie():
    p = create_project("test13")
    expected_data = np.ones((10, 10))
    p.store(expected_data, "data/test_data")
    p.link(data_path, "data2", recursive=True)
    output_path = export_project("test13")
    assert output_path.exists()
    import_project(output_path, "test_import", verbose=True)
    p2 = load_project("test_import")
    # get the list of folders in this path
    data = p2.get("data/test_data")
    assert np.all(data == expected_data)
