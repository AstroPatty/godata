from godata_lib import project
manager = project.ProjectManager()
opened_projects = {}


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