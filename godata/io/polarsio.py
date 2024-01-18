from pathlib import Path

import polars as pl


def get_pd_writer(type_: pl.DataFrame):
    def write_polars_csv(df: pl.DataFrame, path: Path, **kwargs):
        df.write_csv(path, **kwargs)

    write_polars_csv.__sufix__ = ".csv"
    return write_polars_csv


def get_pd_reader(suffix=".csv") -> pl.DataFrame:
    def read_polars_csv(path: Path, **kwargs):
        return pl.read_csv(path, **kwargs)

    return read_polars_csv
