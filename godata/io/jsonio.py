import json


def get_json_writer(type_: dict):
    def write_json(data: dict, path: str, **kwargs):
        json.dump(data, open(path, "w"), **kwargs)

    write_json.__sufix__ = ".json"
    return write_json


def get_json_reader(suffix=".json") -> dict:
    def read_json(path: str, **kwargs):
        return json.load(open(path, "r"), **kwargs)

    return read_json
