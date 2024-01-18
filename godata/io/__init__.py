import importlib
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


def get_typekey(cls):
    class_name = cls.__name__
    module_name = cls.__module__
    type_key = f"{module_name}.{class_name}"
    return type_key


for loader, module_name, is_pkg in pkgutil.walk_packages(__path__):
    try:
        spec = loader.find_spec(module_name)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
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
                f_ = child(par.annotation)
                key = get_typekey(par.annotation)
                if key in _known_writers:
                    _known_writers[key].append(f_)
                else:
                    _known_writers.update({key: [f_]})
            elif "read" in child.__name__:
                if len(signature.parameters) != 1:
                    raise ValueError("Reader functions must take exactly one argument")
                par = list(signature.parameters.values())[0]
                return_type = signature.return_annotation
                if return_type == inspect._empty:
                    raise ValueError(
                        "Reader functions must have a return type annotation"
                    )

                return_type = get_typekey(signature.return_annotation)
                if par.default == inspect._empty:
                    raise ValueError(
                        "Reader functions must a default value for their "
                        "argument, which specifies a file suffix."
                    )
                elif not isinstance(par.default, str):
                    raise ValueError("Reader function annotations must be types")

                if (key := par.default) in _known_readers:
                    _known_readers[key].append((child(), return_type))

                else:
                    _known_readers.update({key: [(child(), return_type)]})


def get_known_readers():
    return _known_readers


def get_known_writers():
    return _known_writers


def try_to_read(path: Path, obj_type: type = None, reader_kwargs: dict = {}):
    readers = get_known_readers()
    suffix = path.suffix
    if suffix not in readers:
        raise godataIoException(f"No reader found for file type {suffix}")
    if obj_type is None:
        reader_fn = readers[suffix][0][0]

    else:
        for reader, type_ in readers[suffix]:
            if type_ == obj_type:
                reader_fn = reader
                break
        else:
            raise godataIoException(
                f"No reader found for file type {suffix} and object type {obj_type}."
                "Perhaps you need to install a library?"
            )

    return reader_fn(path, **reader_kwargs)


def find_writer(obj, format: str = None):
    obj_key = get_typekey(type(obj))
    writers = get_known_writers()
    if obj_key not in writers:
        raise godataIoException(f"No writer found for object type {obj_key}")
    available_writers = writers[obj_key]
    if len(available_writers) == 1 or not format:
        writer = available_writers[0]
    elif format:
        for write_fn in available_writers:
            if write_fn.__sufix__ == format:
                writer = write_fn
                break

        else:
            raise godataIoException(
                f"No writer found for object type {obj_key} and format {format}"
            )
    suffix = writer.__sufix__
    return writer, suffix


__all__ = ["try_to_read", "find_writer"]
