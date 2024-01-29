from astropy.io import fits


def get_fits_writer(type_: fits.HDUList):
    def write_fits(data: fits.HDUList, path: str, **kwargs):
        data.writeto(path, **kwargs)

    write_fits.__sufix__ = ".fits"
    return write_fits
