import polars as pl


def get_pd_writer(type_: pl.DataFrame):
    f_ = lambda df, path: df.write_csv(path, index=False)
    f_.__sufix__ = ".csv"
    return f_


def get_pd_reader(suffix=".csv") -> pl.DataFrame:
    f_ = lambda path: pl.read_csv(path)
    return f_
