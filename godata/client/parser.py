"""
Small routines for parsing information returned by the API.

Most of the possible errors in godata are going to come from the
server side, so a lot of this is just going to be handling that.
"""
from requests import Response

from godata.errors import AlreadyExists, GodataFileError, GodataProjectError, NotFound


def parse_response(response: Response, request_type: str) -> dict:
    if request_type not in ("file", "project"):
        raise ValueError(f"Invalid request type: {request_type}")

    if response.ok:
        return response.json()

    if request_type == "file":
        error = match_file_error(response.status_code)
    else:
        error = match_project_error(response.status_code)

    raise error(response.json())


def match_file_error(status_code: int):
    match status_code:
        case 403:
            return PermissionError
        case 404:
            return FileNotFoundError
        case 409:
            return
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


FILE_ERROR_MAP = {
    404: FileNotFoundError,
    409: FileExistsError,
}
