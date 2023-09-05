from godata_lib import project
from typing import Any
from pathlib import Path

manager = project.ProjectManager()
opened_projects = {}

__all__ = ["open_project", "list_projects", "create_project"]

class GodataProject:
    """
    This is at thin wrapper class for the associated Project struct in the rust library.
    In general, this class just calls the underlying rust methods. However, it does have
    to provide additional behavior in particular cases. For example, storing a python
    object requires a function that knows how to write the given object to a file, which
    will most likely be in python.

    Note that in most cases error handling is actually done by the rust library, so in
    almost all cases expect that an exception encountered while using this class is 
    coming from there.

    This class also provides docstrings for the underlying methods, such that all
    user-facing documentation can be done with sphinx.
    """

    def __init__(self, _project):
        self._project = _project
    
    def __getattr__(self, name):
        return getattr(self._project, name)


    def remove(self, project_path: str, recursive: bool = False):
        """
        Remove an file/folder at the given path. If a folder contains other 
        files/folders, this will throw an error unless rucursive is set to True.
        """
        self._project.remove(project_path, recursive)

    def get(self, project_path: str):
        """
        Get an object at a given product path. This method will return a python object
        whenever possible. If godata doesn't know how to read in a file of this type,
        it will return a path.
        """
        self._project.get(project_path)


    def store(self, object: Any, project_path: str):
        # We have to find the right store function for this object.
        raise NotImplementedError("Not implemented yet")
    
    def add_file(self, file_path: Path, project_path: str):
        """
        Add a file to the project. This will not actually move any data, just create
        a reference to the file.
        """
        self._project.add_file(file_path, project_path)

    def ls(self, project_path: str = None):
        """
        A basic ls utility for looking at projects. If a path is given, this will
        perform the ls in the folder at the given path. Otherwise, it will perform
        it in the project root.
        """



def create_project(name, collection = None):
    pname = collection or "default" + "." + name
    #Note, the manager will throw an error if the project already exists
    project =  manager.create_project(name, collection)
    opened_projects[pname] = project
    return GodataProject(project)

def open_project(name, collection = None):
    pname = collection or "default" + "." + name
    if pname in opened_projects:
        return opened_projects[pname]

    project = manager.load_project(name, collection)
    opened_projects[pname] = project
    return GodataProject(project)

def list_projects(collection = None):

    projects = manager.list_projects(collection)
    print(f"Projects in collection `{collection or 'default'}`:")
    for p in projects:
        print(f"  {p}")