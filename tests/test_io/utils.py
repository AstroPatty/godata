"""
Tests for handling i/o of various file types
"""
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Callable

from pytest import fixture

from godata.io import find_writer, try_to_read

T = Any
O = Callable[[T, T], bool]


def io_test_builder() -> Callable[[T, Path, O], bool]:
    """
    A helper function that will produce a test function for a given data type.
    This test function will test both the reader and writer for the given data type.

    For the moment, this only tests default implementations.
    """

    def test_io(data: T, temp_dir: Path, assert_fn: O, *args, **kwargs) -> bool:
        writer, suffix = find_writer(data)
        temp_path = temp_dir / f"test{suffix}"
        writer(data, temp_path)
        print(temp_path)
        read_data = try_to_read(temp_path, *args, **kwargs)
        val = assert_fn(data, read_data)
        return assert_fn(data, read_data)

    return test_io
