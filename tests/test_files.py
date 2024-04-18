import os
from pathlib import Path

import numpy as np
import pandas as pd
import polars as pl
import pytest

from godata import create_project
from godata.errors import GodataError, GodataProjectError

data_path = Path(os.environ.get("DATA_PATH"))


@pytest.fixture(scope="module")
def project():
    p = create_project("test_files")
    return p


def test_add_file(project):
    project.link(data_path / "test_ones.npy", "data/test_data")
    items = project.get("data/test_data")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)


def test_add_folder(project):
    project.link(data_path, "data")
    items = project.get("data/test_ones.npy")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)


def test_add_with_metadata(project):
    project.link(
        data_path / "test_ones.npy", "data/test_meta", metadata={"test": "test"}
    )
    items = project.get("data/test_meta")
    expected_data = np.ones((10, 10))
    expected_metadata = {"test": "test"}
    path = project.get("data/test_meta", as_path=True)
    expected_metadata["real_path"] = str(path)
    assert np.all(items == expected_data)
    found_metadata = project.get_metadata("data/test_meta")
    assert found_metadata == expected_metadata


def test_move(project):
    data = np.random.rand(10, 10)
    project.store(data, "data/test_move_data")
    project.move("data/test_move_data", "data_moved/test_move_data2")
    project.ls("data_moved")
    found_data = project.get("data_moved/test_move_data2")
    assert np.all(found_data == data)
    assert not project.has_path("data/test_move_data")


def test_move_folder(project):
    data1 = np.random.rand(10, 10)
    data2 = np.random.rand(10, 10)

    project.store(data1, "move_folder/test_data")
    project.store(data2, "move_folder/test_data2")

    project.move("move_folder", "moved_folder")
    project.ls()
    found_data1 = project.get("moved_folder/test_data")
    found_data2 = project.get("moved_folder/test_data2")
    assert np.all(found_data1 == data1)
    assert np.all(found_data2 == data2)
    assert not project.has_path("move_folder")


def test_store_file(project):
    expected_data = np.random.rand(10, 10)
    project.store(expected_data, "data/test_stored_data")
    items = project.get("data/test_stored_data")
    assert np.all(items == expected_data)
    metadata = project.get_metadata("data/test_stored_data")
    assert metadata["obj_type"] == "numpy.ndarray"


def test_store_in_root(project):
    expected_data = np.random.rand(10, 10)
    project.store(expected_data, "test_stored_data")
    items = project.get("test_stored_data")
    assert np.all(items == expected_data)
    metadata = project.get_metadata("test_stored_data")
    assert metadata["obj_type"] == "numpy.ndarray"


def test_overwrite(project):
    expected_data = np.random.rand(10, 10)
    project.store(expected_data, "data/test_data_overwrite")
    stored_path = project.get("data/test_data_overwrite", as_path=True)

    df_data = pd.read_csv(data_path / "test_df.csv")
    with pytest.raises(FileExistsError):
        project.store(df_data, "data/test_data_overwrite")
    project.store(df_data, "data/test_data_overwrite", overwrite=True)
    data = project.get("data/test_data_overwrite")
    assert np.all(data.values == df_data.values)
    assert not stored_path.exists()


def test_multiple_get(project):
    expected_data = np.random.rand(10, 10)
    expected_data2 = np.random.rand(10, 10)
    project.store(expected_data, "new_data/test_data.npy")
    project.store(expected_data2, "new_data/test_data2.npy")
    project.store(expected_data, "new_data/test_data3")

    results = project.get_many("new_data", "*.npy")
    assert len(results) == 2
    assert np.all(results["test_data.npy"] == expected_data)
    assert np.all(results["test_data2.npy"] == expected_data2)


def test_store_different_type(project):
    df_data = pd.read_csv(data_path / "test_df.csv")
    project.store(df_data, "data/test_data_parquet", format=".parquet")
    stored_path = project.get("data/test_data_parquet", as_path=True)
    assert stored_path.suffix == ".parquet"
    read_df = pd.read_parquet(stored_path)
    assert np.all(read_df.values == df_data.values)
    read_df = project.get("data/test_data_parquet")
    assert np.all(read_df.values == df_data.values)


def test_get_different_type(project):
    df_data = pd.read_csv(data_path / "test_df.csv")
    project.store(df_data, "data/test_data_transform")
    stored_data = project.get("data/test_data_transform", load_type=pl.DataFrame)
    assert isinstance(stored_data, pl.DataFrame)
    stored_data = project.get("data/test_data_transform")
    assert isinstance(stored_data, pd.DataFrame)


def test_multiple_get_no_files(project):
    with pytest.raises(FileNotFoundError):
        _ = project.get_many("data", "*.txt")


def test_store_with_kwargs(project):
    df_data = pd.read_csv(data_path / "test_df.csv")
    project.store(df_data, "data/test_data_windex", writer_kwargs={"index": True})
    stored_path = project.get("data/test_data_windex", as_path=True)
    read_df = pd.read_csv(stored_path)
    assert "Unnamed: 0" in read_df.columns and "Unnamed: 0" not in df_data.columns
    # This function will fail if defaults for pandas change


def test_read_with_kwargs(project):
    data = project.get("data/test_data_windex", reader_kwargs={"nrows": 2})
    assert len(data) == 2


def test_exists(project):
    expected_data = np.random.rand(10, 10)
    project.store(expected_data, "data/test_exists_data")
    hp1 = project.has_path("data/test_exists_data")
    hp2 = project.has_path("data/test_data2_doesnt_exist")
    # Check the json parsing as well
    assert hp1 and not hp2


def test_invalid_path_fnf(project):
    with pytest.raises(FileNotFoundError):
        project.get("data/some_random_path")


def test_delete(project):
    expected_data = np.random.rand(10, 10)
    project.store(expected_data, "delete_single_data/test_delete_data")
    project.store(expected_data, "delete_single_data2/test_delete_data")

    path = project.get("delete_single_data/test_delete_data", as_path=True)
    path2 = project.get("delete_single_data2/test_delete_data", as_path=True)
    project.remove("delete_single_data/test_delete_data")
    hp1 = project.has_path("delete_single_data/test_delete_data")
    children = project.list()
    assert not hp1 and "delete_single_data" not in children["folders"]
    assert not path.exists()

    project.remove("delete_single_data2")
    hp2 = project.has_path("delete_single_data2")
    children = project.list()
    assert not hp2 and "delete_single_data2" not in children["folders"]
    assert not path2.exists()


def test_delete_link(project):
    project.link(data_path / "test_ones.npy", "data_delete/test_delete_link")

    path = project.get("data_delete/test_delete_link", as_path=True)

    project.remove("data_delete/test_delete_link")
    hp1 = project.has_path("data_delete/test_delete_link")
    children = project.list()
    assert not hp1 and "data_delete" not in children["folders"]

    assert path.exists()


def test_project_path_clean(project):
    expected_data = np.random.rand(10, 10)
    project.store(expected_data, "/data/test_path_clean_data/")
    data = project.get("/data/test_path_clean_data/")
    data2 = project.get("data/test_path_clean_data/")
    assert np.all(data == data2)
    assert np.all(data == expected_data)
