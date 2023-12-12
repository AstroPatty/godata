import numpy as np
import pandas as pd
import pytest

from godata import create_project
from godata.project import GodataProjectError


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
    p.link("/home/data/test_ones.npy", "data/test_data")
    items = p.get("data/test_data")
    expected_data = np.ones((10, 10))
    assert np.all(items == expected_data)


def test_add_folder():
    p = create_project("test4")
    p.link("/home/data", "data")
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

    df_data = pd.read_csv("/home/data/test_df.csv")
    p.store(df_data, "data/test_data")
    data = p.get("data/test_data")
    assert np.all(data.values == df_data.values)
    assert not stored_path.exists()
