import pandas as pd


def get_pd_writer(type_: pd.DataFrame):
    f_ = lambda df, path: df.to_csv(path, index=False)
    return f_


def get_pd_reader(suffix="csv") -> pd.DataFrame:
    f_ = lambda path: pd.read_csv(path)
    return f_
