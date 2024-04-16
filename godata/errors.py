from typing import Optional, Protocol


class GodataError(Protocol):
    def to_native_error(self) -> Optional[Exception]:
        """
        Return a native Python exception (like FileNotFound) that
        corresponds to this error, or None if there isn't one.
        """
        pass
