from os import environ
from pathlib import Path
from tempfile import TemporaryDirectory

import polars as pl
from pytest import fixture

from godata.io import get_typekey

from .utils import io_test_builder


@fixture
def temp_dir():
    with TemporaryDirectory() as temp_dir:
        yield Path(temp_dir)


def test_polars_io(temp_dir):
    data_path = Path(environ.get("DATA_PATH"))
    data = pl.read_csv(data_path / "test_df.csv")
    test_io = io_test_builder()
    typekey = get_typekey(pl.DataFrame)
    assert_fn = lambda x, y: x.equals(y)
    assert test_io(data, temp_dir, assert_fn, obj_type=typekey)
