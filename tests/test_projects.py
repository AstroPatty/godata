import uuid
from pathlib import Path

import pytest

from godata.project import GodataProject


def test_add(project_name):
    from godata import list_projects, open_project

    text_file_location = Path(__file__).parent / "test.txt"
    project = open_project(project_name)
    project.add_file(str(text_file_location), "test_file")
    file = project.get("test_file")
    assert file == str(text_file_location)


def test_add_in_folder(project_name):
    from godata import open_project

    text_file_location = Path(__file__).parent / "test.txt"
    project = open_project(project_name)
    project.add_file(str(text_file_location), "folder/test_file")
    file = project.get("folder/test_file")
    assert file == str(text_file_location)


@pytest.fixture
def project_name():
    from godata.project import remove_project

    name = str(uuid.uuid4())
    yield name


@pytest.fixture(autouse=True)
def project(project_name):
    from godata import create_project, remove_project

    yield create_project(project_name)
    remove_project(project_name)
