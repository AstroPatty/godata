from functools import cache
from pathlib import Path
from urllib import parse

import requests
from packaging import version
from requests.adapters import HTTPAdapter

from godata import server

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
    pass


class AlreadyExists(Exception):
    pass


class Forbidden(Exception):
    pass


class GodataClientError(Exception):
    pass


def check_server(client, url):
    from godata import __minimum_server_version__

    server_version = version.parse(get_version(client, url))
    minium_version = version.parse(__minimum_server_version__)

    if server_version < minium_version:
        raise GodataClientError(
            f"Server version {server_version} is less than minimum version "
            f"{minium_version}. Please upgrade the server."
        )
    return True


@cache
def get_client():
    try:
        with open(Path.home() / ".godata_server", "r") as f:
            SERVER_URL = f.read().strip()
    except FileNotFoundError:
        SERVER_URL = None

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
    if resp.status_code == 200:
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
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


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
    if result.status_code == 201:
        return result.json()
    elif result.status_code == 409:
        raise AlreadyExists(f"{result.json()}")
    else:
        raise GodataClientError(f"{result.status_code}: {result.text}")


def delete_project(collection_name: str, project_name: str, force: bool = False):
    client, url = get_client()
    payload = {"force": str(force).lower()}
    resp = client.delete(
        f"{url}/projects/{collection_name}/{project_name}", params=payload
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    elif resp.status_code == 403:
        raise Forbidden(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def load_project(collection_name: str, project_name: str):
    """
    Load a project into the server memory if it is not already loaded.
    """
    client, url = get_client()
    resp = client.post(f"{url}/load/{collection_name}/{project_name}")
    if resp.status_code == 200:
        print(resp.json())
        return True
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


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
    if resp.status_code == 200:
        return resp.json()
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def path_exists(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/exists", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


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
        raise GodataClientError(
            f"Metadata keys {set(metadata.keys())} conflict with parameter keys "
            f"{set(params.keys())}."
        )
    for k, v in metadata.items():
        try:
            params.update({str(k): str(v)})
        except TypeError:
            raise GodataClientError(
                "Metadata keys and values must be convertible strings."
            ) from None

    resp = client.post(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    if resp.status_code == 201:
        return resp.json()
    elif resp.status_code == 409:
        raise AlreadyExists(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


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
    if resp.status_code == 201:
        return resp.json()
    elif resp.status_code == 409:
        raise AlreadyExists(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


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
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def get_file(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    else:
        raise NotFound(f"{resp.json()}")


def generate_path(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{url}/projects/{collection_name}/{project_name}/generate",
        params=params,
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def remove_file(collection_name: str, project_name: str, project_path: str):
    client, url = get_client()
    params = {"project_path": project_path}
    resp = client.delete(
        f"{url}/projects/{collection_name}/{project_name}/files", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def export_tree(collection_name: str, project_name: str, output_path: Path):
    client, url = get_client()
    params = {"output_path": str(output_path)}
    resp = client.get(f"{url}/export/{collection_name}/{project_name}", params=params)
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def import_tree(collection_name: str, project_name: str, input_path: Path):
    client, url = get_client()
    params = {"input_path": str(input_path)}
    resp = client.get(f"{url}/import/{collection_name}/{project_name}", params=params)
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")
