from typing import Optional, Protocol


class GodataErrorType(Protocol):
    def to_native_error(self) -> Optional[Exception]:
        """
        Return a native Python exception (like FileNotFound) that
        corresponds to this error, or None if there isn't one.
        """
        pass


class GodataError(Exception):
    pass


class GodataProjectError(Exception):
    pass


class AlreadyExists(GodataProjectError):
    pass


class NotFound(GodataProjectError):
    pass


class GodataFileError(Exception):
    pass
