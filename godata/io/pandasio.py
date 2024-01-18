import pandas as pd


def get_pd_csv_writer(type_: pd.DataFrame):
    f_ = lambda df, path: df.to_csv(path, index=False)
    f_.__sufix__ = ".csv"
    return f_


def get_pd_csv_reader(suffix=".csv") -> pd.DataFrame:
    f_ = lambda path: pd.read_csv(path)
    return f_


def get_pd_parquet_writer(type_: pd.DataFrame):
    f_ = lambda df, path: df.to_parquet(path, index=False)
    f_.__sufix__ = ".parquet"
    return f_


def get_pd_parquet_reader(suffix=".parquet") -> pd.DataFrame:
    f_ = lambda path: pd.read_parquet(path)
    return f_
