import os
from pathlib import Path


def handle_overwrite(link_result: dict):
    """
    This function handles the case when a file has been overwritten in the project
    and should be delted from disk. This happens whenver a file that was stored in
    the project is overwritten.
    """
    overwritten_files = link_result["overwritten"]
    if overwritten_files != "none":
        handle_removal(overwritten_files)


def handle_removal(to_remove: list[str]) -> None:
    """
    This function handles the case when a file has been removed from the project
    and should be delted from disk. This happens whenver a file that was stored in
    the project is removed.
    """
    paths = [Path(file) for file in to_remove]
    files_by_folder = {}
    for path in paths:
        if path.is_dir():
            continue
        folder = path.parent
        if folder not in files_by_folder:
            files_by_folder[folder] = []
        files_by_folder[folder].append(path)
    for folder, files in files_by_folder.items():
        for file in files:
            os.remove(file)
        if not os.listdir(folder):
            os.rmdir(folder)
