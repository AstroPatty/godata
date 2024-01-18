from pathlib import Path

import pandas as pd


def get_pd_csv_writer(type_: pd.DataFrame):
    def pandas_csv_writer(df: pd.DataFrame, path: Path, **kwargs):
        if "index" not in kwargs:
            kwargs["index"] = False
        df.to_csv(path, **kwargs)

    pandas_csv_writer.__sufix__ = ".csv"
    return pandas_csv_writer


def get_pd_csv_reader(suffix=".csv") -> pd.DataFrame:
    def pandas_csv_reader(path: Path, **kwargs):
        return pd.read_csv(path, **kwargs)

    return pandas_csv_reader


def get_pd_parquet_writer(type_: pd.DataFrame):
    def pandas_parquet_writer(df: pd.DataFrame, path: Path, **kwargs):
        if "index" not in kwargs:
            kwargs["index"] = False
        df.to_parquet(path, **kwargs)

    pandas_parquet_writer.__sufix__ = ".parquet"
    return pandas_parquet_writer


def get_pd_parquet_reader(suffix=".parquet") -> pd.DataFrame:
    def pandas_parquet_reader(path: Path, **kwargs):
        return pd.read_parquet(path, **kwargs)

    return pandas_parquet_reader
