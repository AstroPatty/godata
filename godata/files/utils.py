import os


def handle_overwrite(link_result: dict):
    """
    This function handles the case when a file has been overwritten in the project
    and should be delted from disk. This happens whenver a file that was stored in
    the project is overwritten.
    """
    overwritten_file = link_result["overwritten"]
    if overwritten_file != "none":
        os.remove(overwritten_file)
        print(f"Removed {overwritten_file} from disk")
