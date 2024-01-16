def split_name(name: str) -> tuple:
    split = name.split("/")
    if len(split) == 1:
        project_name = split[0]
        collection = "default"
    elif len(split) == 2:
        project_name = split[1]
        collection = split[0]
    else:
        raise ValueError("Invalid project name.")
    return project_name, collection
