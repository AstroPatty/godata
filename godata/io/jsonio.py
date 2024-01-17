import json


def get_json_writer(type_: dict):
    f_ = lambda data, path: json.dump(data, open(path, "w"))
    f_.__sufix__ = ".json"
    return f_


def get_json_reader(suffix=".json") -> dict:
    f_ = lambda path: json.load(open(path, "r"))
    return f_
