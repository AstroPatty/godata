import numpy as np


def get_np_writer(type_: np.ndarray):
    f_ = lambda array, path: np.save(path, array)
    return f_


def get_np_reader(suffix="npy") -> np.ndarray:
    f_ = lambda path: np.load(path)
    return f_
