import inspect
from typing import Callable, ParamSpec, TypeVar

T = TypeVar("T")
P = ParamSpec("P")
# Signature variable


def sanitize_project_path(func: Callable[P, T]) -> Callable[P, T]:
    """
    A decorator to sanitize project paths. This will convert any argument
    named "project_path" to a string, and remove any leading or trailing slashes.

    This ensures anyhting that gets passed to the underlying project is of the
    correct form
    """

    def wrapper(*args, **kwargs) -> T:
        # Get the signature of the function

        if len(args) == 0 and len(kwargs) == 0:
            return func()
        sig = inspect.signature(func)
        # Figure out where the project_path argument is
        arg_index = None
        for i, name in enumerate(sig.parameters.keys()):
            if name == "project_path":
                arg_index = i

        if not arg_index:
            raise ValueError(
                f"No argument named 'project_path' found in function {func.__name__}"
            )
        project_path = None
        if len(args) > arg_index:
            # If the argument is passed as a positional argument
            project_path = args[arg_index]

        elif "project_path" in kwargs:
            # If the argument is passed as a keyword argument
            project_path = kwargs["project_path"]

        if project_path is not None and isinstance(project_path, str):
            if project_path.startswith("/"):
                # remove the leading slash
                project_path = project_path[1:]
            if project_path.endswith("/"):
                # remove the trailing alsh
                project_path = project_path[:-1]

        if arg_index is not None and arg_index < len(args):
            # If the argument is passed as a positional argument
            args = list(args)
            args[arg_index] = project_path
        else:
            # If the argument is passed as a keyword argument
            kwargs["project_path"] = project_path
        return func(*args, **kwargs)

    return wrapper
