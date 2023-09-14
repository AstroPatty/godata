import inspect
import pkgutil
from pathlib import Path

_known_writers = {}
_known_readers = {}
_io_match = {}

"""


In general, we do not want godata to depend on every single package for
which we may provide input/output functionality. To facilitate this, i/o options are
seperated into individual files. If the library in question is not installed on the
user's machine, we skip over it.

Functions to get a given reader or writer must have the name "read" or "write" in them,
respectively.

Writers should take in a single argument with a type annotation. This type annotation
should be the type of an object the writer knows how to write. The "get_writer" function
(or whatever it's named) should return a function that takes in two arguments: the
object to write and a path to write to, in that order.

Readers should take in a single argument of type string, with the default value being
the suffix of the file that will be delegated to this reader. The "get_reader" function
(or whatever it's named) should return a function that takes in a single argument: the
path to the file to read. This function should return the object that was read in.
"""


class godataIoException(Exception):
    pass


for loader, module_name, is_pkg in pkgutil.walk_packages(__path__):
    try:
        module = loader.find_module(module_name).load_module(module_name)
    except ImportError:
        continue
    children = [getattr(module, child) for child in dir(module)]
    for child in children:
        if hasattr(child, "__call__") and not child.__name__.startswith("__"):
            signature = inspect.signature(child)
            if "write" in child.__name__:
                if len(signature.parameters) != 1:
                    raise ValueError("Writer functions must take exactly one argument")
                par = list(signature.parameters.values())[0]
                if par.annotation == inspect._empty:
                    raise ValueError("Writer functions must have type annotations")
                elif not isinstance(par.annotation, type):
                    raise ValueError("Writer function annotations must be types")
                elif par.annotation in _known_writers:
                    raise ValueError(
                        f"Writer function already registered for type "
                        f"{par.annotation.__name__}"
                    )
                f_ = child(par.annotation)
                _known_writers.update({par.annotation: f_})
            elif "read" in child.__name__:
                if len(signature.parameters) != 1:
                    raise ValueError("Reader functions must take exactly one argument")
                par = list(signature.parameters.values())[0]
                return_type = signature.return_annotation
                if return_type == inspect._empty:
                    raise ValueError(
                        "Reader functions must have a return type annotation"
                    )
                if par.default == inspect._empty:
                    raise ValueError(
                        "Reader functions must a default value for their "
                        "argument, which specifies a file suffix."
                    )
                elif not isinstance(par.default, str):
                    raise ValueError("Reader function annotations must be types")
                elif par.default.strip(".") in _known_readers:
                    raise ValueError(
                        f"Reader function already registered for file type "
                        f"{par.annotation.__name__}"
                    )

                f_ = child()
                _known_readers.update({par.default.strip("."): (f_, return_type)})

for suffix, (f_, rtype) in _known_readers.items():
    if rtype in _known_writers.keys():
        writer_function = _known_writers[rtype]
        _known_writers.update({rtype: (writer_function, suffix)})
    else:
        raise ValueError(f"Reader functions must have an equivalent writer function!")


def get_known_readers():
    return _known_readers


def get_known_writers():
    return _known_writers


def try_to_read(path: Path):
    readers = get_known_readers()
    suffix = path.suffix.strip(".")
    if suffix not in readers:
        raise godataIoException(f"No reader found for file type {suffix}")
    reader_fn = readers[suffix][0]
    return reader_fn(path)


__all__ = ["get_known_readers", "get_known_writers"]
