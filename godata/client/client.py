from functools import cache
from pathlib import Path
from typing import Optional
from urllib import parse

import requests
from packaging import version
from requests.adapters import HTTPAdapter

from godata import server
from godata.errors import GodataError

from .parser import RequestType, parse_response
from .unixsocket import UnixHTTPAdapter

"""
The client connects to the godata server and communicates with it on behalf of the 
project. On Mac and Linux, this communication is done via a unix socket with a 
REST-ish API. Windows is not supported at this time, but will probalby just have 
to use a TCP socket instead of a unix socket.

The client is stateless, so it's just a bunch of functions.

I need to think a bit about how to properly reuse the client sesion.
"""


class NotFound(Exception):
    def to_native_error(self):
        return FileNotFoundError(str(self)).with_traceback(self.__traceback__)


class AlreadyExists(Exception):
    pass


class Forbidden(Exception):
    pass


def check_server(client, url):
    from godata import __minimum_server_version__

    server_version = version.parse(get_version(client, url))
    minium_version = version.parse(__minimum_server_version__)

    if server_version < minium_version:
        raise GodataError(
            f"Server version {server_version} is less than minimum version "
            f"{minium_version}. Please upgrade the server."
        )
    return True


@cache
def get_client():
    server_config = server.get_config()
    SERVER_URL = server_config.server_url

    if not SERVER_URL:
        server.start()
        return get_client()

    elif SERVER_URL.startswith("http+unix://"):
        SERVER_PATH = parse.unquote(SERVER_URL.split("://")[1])
        ADAPTER = UnixHTTPAdapter(SERVER_PATH)
    else:
        ADAPTER = HTTPAdapter()

    CLIENT = requests.Session()
    CLIENT.mount(SERVER_URL, ADAPTER)

    try:
        check_server(CLIENT, SERVER_URL)
        return (CLIENT, SERVER_URL)
    except requests.exceptions.ConnectionError:
        server.start()
        return get_client()


def get_version(client, url):
    resp = client.get(f"{url}/version")
    return resp.json()


def list_collections(show_hidden=False):
    client, url = get_client()
    payload = {"show_hidden": str(show_hidden).lower()}
    result = client.get(f"{url}/collections", params=payload)
    return result.json()


def list_projects(collection_name: str, show_hidden: bool = False):
    client, url = get_client()
    payload = {"show_hidden": str(show_hidden).lower()}
    resp = client.get(f"{url}/projects/{collection_name}", params=payload)
    return parse_response(resp, RequestType.PROJECT)


def create_project(
    collection_name: str,
    project_name: str,
    force: bool = False,
    storage_location: str = None,
):
    client, url = get_client()
    args = {"force": str(force).lower()}
    if storage_location:
        args["storage_location"] = storage_location
    result = client.post(f"{url}/create/{collection_name}/{project_name}", params=args)
    return parse_response(result, RequestType.PROJECT)


def delete_project(collection_name: str, project_name: str, force: bool = False):
    client, url = get_client()
    payload = {"force": str(force).lower()}
    resp = client.delete(
        f"{url}/projects/{collection_name}/{project_name}", params=payload
    )
    return parse_response(resp, RequestType.PROJECT)


def load_project(collection_name: str, project_name: str):
    """
    Load a project into the server memory if it is not already loaded.
    """
    client, url = get_client()
    resp = client.post(f"{url}/load/{collection_name}/{project_name}")
    if resp.status_code == 200:
        print(resp.json())
        return True
    else:
        return parse_response(resp, RequestType.PROJECT)


def drop_project(collection_name: str, project_name: str):
    """
    Signals to the server this client is done with this project. This may or may not
    actually drop the project from memory, depending on if other clients are using it.
    """
    client, url = get_client()
    try:
        resp = client.post(f"{url}/drop/{collection_name}/{project_name}")
    except requests.exceptions.ConnectionError:
        # The server is probably down, so this operation doesn't really matter
        return {}
    return parse_response(resp, RequestType.PROJECT)


def path_exists(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/exists", params=params
    )
    return parse_response(resp, RequestType.FILE)


def link_file(
    collection_name: str,
    project_name: str,
    project_path: str,
    file_path: str,
    metadata: dict = {},
    force: bool = False,
):
    client, url = get_client()
    params = {
        "project_path": project_path,
        "real_path": file_path,
        "force": str(force).lower(),
    }
    if set(metadata.keys()).intersection(set(params.keys())):
        raise GodataError(
            f"Metadata keys {set(metadata.keys())} conflict with parameter keys "
            f"{set(params.keys())}."
        )
    for k, v in metadata.items():
        try:
            params.update({str(k): str(v)})
        except TypeError:
            raise GodataError(
                "Metadata keys and values must be convertible strings."
            ) from None

    resp = client.post(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    result = parse_response(resp, RequestType.FILE)
    print(result)
    return result


def link_folder(
    collection_name: str,
    project_name: str,
    project_path: str,
    folder_path: str,
    recursive: bool = False,
):
    client, url = get_client()
    params = {
        "project_path": project_path,
        "real_path": folder_path,
        "type": "folder",
        "recursive": str(recursive).lower(),
    }
    resp = client.post(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    return parse_response(resp, RequestType.FILE)


def move(
    collection_name: str,
    project_name: str,
    source_path: str,
    destination_path: str,
    overwrite: bool = False,
):
    client, url = get_client()
    params = {
        "source_path": source_path,
        "destination_path": destination_path,
        "overwrite": str(overwrite).lower(),
    }
    resp = client.post(
        f"{url}/projects/{collection_name}/{project_name}/files/move", params=params
    )
    return parse_response(resp, RequestType.FILE)


def list_project_contents(
    collection_name: str,
    project_name: str,
    project_path=None,
    show_hidden: bool = False,
):
    client, url = get_client()
    params = {"show_hidden": str(show_hidden).lower()}
    if project_path:
        params["project_path"] = project_path

    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/list", params=params
    )
    return parse_response(resp, RequestType.FILE)


def get_file(
    collection_name: str,
    project_name: str,
    project_path: Optional[str] = None,
    pattern: Optional[str] = None,
):
    client, url = get_client()
    params = {}
    if project_path:
        params["project_path"] = project_path
    if pattern:
        params["pattern"] = pattern
    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    return parse_response(resp, RequestType.FILE)


def generate_path(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/generate",
        params=params,
    )
    return parse_response(resp, RequestType.FILE)


def remove_file(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.delete(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    return parse_response(resp, RequestType.FILE)


def export_tree(collection_name: str, project_name: str, output_path: Path):
    client, url = get_client()
    params = {"output_path": str(output_path)}
    resp = client.get(f"{url}/export/{collection_name}/{project_name}", params=params)
    return parse_response(resp, RequestType.PROJECT)


def import_tree(collection_name: str, project_name: str, input_path: Path):
    client, url = get_client()
    params = {"input_path": str(input_path)}
    resp = client.get(f"{url}/import/{collection_name}/{project_name}", params=params)
    return parse_response(resp, RequestType.PROJECT)
