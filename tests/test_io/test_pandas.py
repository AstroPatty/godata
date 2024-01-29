from os import environ
from pathlib import Path
from tempfile import TemporaryDirectory

import pandas as pd
from pytest import fixture

from .utils import io_test_builder


@fixture
def temp_dir():
    with TemporaryDirectory() as temp_dir:
        yield Path(temp_dir)


def test_io_pandas(temp_dir):
    data_path = Path(environ.get("DATA_PATH"))
    data = pd.read_csv(data_path / "test_df.csv")
    test_io = io_test_builder()
    assert_fn = lambda x, y: x.equals(y)
    test_io(data, temp_dir, assert_fn)
