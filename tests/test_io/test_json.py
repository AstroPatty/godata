import json
from os import environ
from pathlib import Path
from tempfile import TemporaryDirectory

from pytest import fixture

from godata.io import get_typekey

from .utils import io_test_builder


@fixture
def temp_dir():
    with TemporaryDirectory() as temp_dir:
        yield Path(temp_dir)


def test_io_json(temp_dir):
    data_path = Path(environ.get("DATA_PATH"))
    data = json.load(open(data_path / "test_json.json"))
    test_io = io_test_builder()
    assert_fn = lambda x, y: x == y
    assert test_io(data, temp_dir, assert_fn)
