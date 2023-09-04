from godata_lib import project
from typing import Any


manager = project.ProjectManager()
opened_projects = {}

__all__ = ["open_project", "list_projects", "create_project"]

class GodataProject:
    def __init__(self, _project):
        self._project = _project
    
    def __getattr__(self, name):
        return getattr(self._project, name)
    

    def store(self, object: Any, project_path: str):
        # We have to find the right store function
        raise NotImplementedError("Not implemented yet")

def create_project(name, collection = None):
    pname = collection or "default" + "." + name
    #Note, the manager will throw in case the project already exists
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