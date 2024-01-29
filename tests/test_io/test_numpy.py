from pathlib import Path
from tempfile import TemporaryDirectory

from pytest import fixture

from .utils import io_test_builder


@fixture
def temp_dir():
    with TemporaryDirectory() as temp_dir:
        yield Path(temp_dir)


def test_io_numpy(temp_dir):
    import numpy as np

    data = np.ones((10, 10))
    test_io = io_test_builder()
    assert_fn = lambda x, y: np.all(x == y)

    test_io(data, temp_dir, assert_fn)
