import numpy as np


def get_np_writer(type_: np.ndarray):
    def write_numpy_array(array: np.ndarray, path: str, **kwargs):
        np.save(path, array, **kwargs)

    write_numpy_array.__sufix__ = ".npy"
    return write_numpy_array


def get_np_reader(suffix=".npy") -> np.ndarray:
    def read_numpy_array(path: str, **kwargs):
        return np.load(path, **kwargs)

    return read_numpy_array
