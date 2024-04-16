import numpy as np
import pytest

from godata import create_project


@pytest.fixture(scope="module")
def project():
    p = create_project("test_files")
    return p


def test_multiple_get(project):
    expected_data = np.ones((10, 10))
    expected_data2 = np.zeros((10, 10))
    project.store(expected_data, "data/test_data.npy")
    project.store(expected_data2, "data/test_data2.npy")
    project.store(expected_data, "data/test_data3")

    results = project.get_many("data", "*.npy")
    assert len(results) == 2
    assert np.all(results["test_data.npy"] == expected_data)
    assert np.all(results["test_data2.npy"] == expected_data2)


def test_multiple_get_no_files(project):
    with pytest.raises(FileNotFoundError):
        _ = project.get_many("data", "*.txt")
