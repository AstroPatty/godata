"""
Small routines for parsing information returned by the API.

Most of the possible errors in godata are going to come from the
server side, so a lot of this is just going to be handling that.
"""
from __future__ import annotations

from enum import Enum

from requests import Response

from godata.errors import (
    AlreadyExists,
    GodataError,
    GodataFileError,
    GodataProjectError,
    NotFound,
)


class RequestType(Enum):
    FILE = 1
    PROJECT = 2
    OTHER = 3


def parse_response(
    response: Response, request_type: RequestType, err_ok: bool = False
) -> dict:
    if response.ok:
        return response.json()
    match request_type:
        case RequestType.FILE:
            error = match_file_error(response.status_code)
        case RequestType.PROJECT:
            error = match_project_error(response.status_code)
        case _:
            error = match_other_error(response.status_code)
    if not err_ok:
        raise error(response.json())


def match_file_error(status_code: int):
    match status_code:
        case 403:
            return PermissionError
        case 404:
            return FileNotFoundError
        case 409:
            return FileExistsError
        case _:
            return GodataProjectError


def match_project_error(status_code: int):
    match status_code:
        case 404:
            return NotFound
        case 409:
            return AlreadyExists
        case _:
            return GodataFileError


def match_other_error(status_code: int):
    return GodataError
