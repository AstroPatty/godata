from os import environ
from pathlib import Path
from tempfile import TemporaryDirectory

import numpy as np
from astropy.io import fits
from pytest import fixture

from .utils import io_test_builder


@fixture
def temp_dir():
    with TemporaryDirectory() as temp_dir:
        yield Path(temp_dir)


def test_io_fits(temp_dir):
    data_path = Path(environ.get("DATA_PATH"))
    data = fits.open(data_path / "test_fits.fits")
    test_io = io_test_builder()

    def assert_fn(x, y):
        for i in range(len(x)):
            if np.all(x[i].data != y[i].data):
                return False
        return True

    assert test_io(data, temp_dir, assert_fn)
