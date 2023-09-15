import uuid
from pathlib import Path

import numpy as np
import pytest

from godata.io import godataIoException
from godata.project import GodataProject


def test_add(project_name):
    from godata import list_projects, load_project

    text_file_location = Path(__file__).parent / "test.txt"
    project = load_project(project_name)
    project.link(text_file_location, "test_file")
    file = project.get("test_file")
    assert file == text_file_location


def test_add_in_folder(project_name, text_file_location):
    from godata import load_project

    project = load_project(project_name)
    project.link(text_file_location, "folder/test_file")
    file = project.get("folder/test_file")
    assert file == text_file_location


def test_store(project_name):
    from godata import load_project

    project = load_project(project_name)
    data = np.random.rand(10, 10)
    project.store(data, "test_data")
    file = project.get("test_data")
    assert np.allclose(file, data)


def test_store_invalid(project_name, text_file_location):
    from godata import load_project

    project = load_project(project_name)
    with pytest.raises(godataIoException):
        project.store(text_file_location, "test_data")


def test_store_valid(project_name, npy_file_location):
    from godata import load_project

    project = load_project(project_name)
    project.store(npy_file_location, "test_data")
    file = project.get("test_data")
    assert np.allclose(file, np.load(npy_file_location))


@pytest.fixture
def project_name():
    name = str(uuid.uuid4())
    yield name


@pytest.fixture(autouse=True)
def project(project_name):
    from godata import create_project, delete_project

    yield create_project(project_name)
    delete_project(project_name)


@pytest.fixture
def text_file_location() -> Path:
    text_file_location = Path(__file__).parent / "test.txt"
    return text_file_location


@pytest.fixture
def npy_file_location() -> Path:
    npy_file_location = Path(__file__).parent / "test_data.npy"
    return npy_file_location
