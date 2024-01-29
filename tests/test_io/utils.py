"""
Tests for handling i/o of various file types
"""
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Callable, TypeVar

from pytest import fixture

from godata.io import find_writer, try_to_read


def io_test_builder():
    """
    A helper function that will produce a test function for a given data type.
    This test function will test both the reader and writer for the given data type.

    For the moment, this only tests default implementations.
    """
    T = TypeVar("T")

    def test_io(data: T, temp_dir: Path, assert_fn: Callable[[T, T], bool]):
        writer, suffix = find_writer(data)
        temp_path = temp_dir / f"test.{suffix}"
        writer(data, temp_path)

        read_data = try_to_read(temp_path)
        assert assert_fn(data, read_data)

    return test_io
