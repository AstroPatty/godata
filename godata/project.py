from godata_lib import project
manager = project.ProjectManager()
opened_projects = {}

__all__ = ["open_project", "list_projects", "create_project"]

def create_project(name, collection = None):
    return manager.create_project(name, collection)

def open_project(name, collection = None):
    if collection is None:
        pname = name
    else:
        pname = collection + "." + name
    if pname in opened_projects:
        return opened_projects[pname]

    project = manager.load_project(name, collection)
    opened_projects[pname] = project
    return project

def list_projects(collection = None):

    projects = manager.list_projects(collection)
    print(f"Projects in collection `{collection or 'default'}`:")
    for p in projects:
        print(f"  {p}")