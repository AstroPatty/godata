from godata import create_project


def test1():
    p = create_project("test1")
    assert p.name == "test1" and p.collection == "default"
