"""
Management of all actual files are handled in python.
"""
from .locking import get_lock

__all__ = ["get_lock"]
