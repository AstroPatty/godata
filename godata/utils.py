import inspect
from functools import wraps
from typing import Callable, ParamSpec, TypeVar

T = TypeVar("T")
P = ParamSpec("P")
# Signature variable


def sanitize_project_path(func: Callable[P, T]) -> Callable[P, T]:
    """
    A decorator to sanitize project paths. This will convert any argument
    with "project_path" in its name into a string, and remove any leading or
    trailing slashes.

    This ensures anyhting that gets passed to the underlying project is of the
    correct form
    """
    sig = inspect.signature(func)
    # Figure out where the project_path argument is
    path_args = [(i, p) for i, p in enumerate(sig.parameters) if "project_path" in p]

    if not path_args:
        raise ValueError(f"No project_path argument found in function {func.__name__}")

    @wraps(func)
    def wrapper(*args, **kwargs) -> T:
        # Get the signature of the function

        if len(args) == 0 and len(kwargs) == 0:
            return func()
        new_args = list(args)
        for arg_index, arg_name in path_args:
            if arg_index < len(args):
                project_path = args[arg_index]
                sanitized_path = sanitize(project_path)
                new_args[arg_index] = sanitized_path
            elif arg_name in kwargs:
                project_path = kwargs[arg_name]
                sanitized_path = sanitize(project_path)
                kwargs[arg_name] = sanitized_path
        return func(*new_args, **kwargs)

    return wrapper


def sanitize(s: str) -> str:
    if s is None:
        return s
    if s.startswith("/"):
        s = s[1:]
    if s.endswith("/"):
        s = s[:-1]
    return s
