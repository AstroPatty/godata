from .project import (
    create_project,
    delete_project,
    has_collection,
    has_project,
    list_collections,
    list_projects,
    load_project,
)

__all__ = [
    "load_project",
    "list_projects",
    "create_project",
    "list_collections",
    "delete_project",
    "has_project",
    "has_collection",
]

__version__ = "0.4.0"

from godata.client import check_server

check_server()
