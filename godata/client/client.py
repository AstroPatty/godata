from pathlib import Path
from urllib import parse

import requests
from packaging import version

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
SERVER_PATH = str(Path.home() / ".godata.sock")
SERVER_URL = f"http+unix://{parse.quote(SERVER_PATH, safe='')}"
CLIENT = requests.Session()
CLIENT.mount("http+unix://", UnixHTTPAdapter(SERVER_PATH))


class NotFound(Exception):
    pass


class AlreadyExists(Exception):
    pass


class Forbidden(Exception):
    pass


class GodataClientError(Exception):
    pass


def check_server():
    from godata import __minimum_server_version__

    server_version = version.parse(get_version(CLIENT))
    minium_version = version.parse(__minimum_server_version__)

    if server_version < minium_version:
        raise GodataClientError(
            f"Server version {server_version} is less than minimum version "
            f"{minium_version}. Please upgrade the server."
        )
    return True


def get_client():
    if not hasattr(get_client, "has_run"):
        try:
            check_server()
        except requests.exceptions.ConnectionError:
            server.start()
            check_server()
        get_client.has_run = True
    return CLIENT


def get_version(client):
    resp = client.get(f"{SERVER_URL}/version")
    if resp.status_code == 200:
        return resp.json()


def list_collections(show_hidden=False):
    client = get_client()
    payload = {"show_hidden": str(show_hidden).lower()}
    result = client.get(f"{SERVER_URL}/collections", params=payload)
    return result.json()


def list_projects(collection_name: str, show_hidden: bool = False):
    client = get_client()
    payload = {"show_hidden": str(show_hidden).lower()}
    resp = client.get(f"{SERVER_URL}/projects/{collection_name}", params=payload)
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
    client = get_client()
    args = {"force": str(force).lower()}
    if storage_location:
        args["storage_location"] = storage_location

    result = client.post(
        f"{SERVER_URL}/create/{collection_name}/{project_name}", params=args
    )
    if result.status_code == 201:
        return result.json()
    elif result.status_code == 409:
        raise AlreadyExists(f"{result.json()}")
    else:
        raise GodataClientError(f"{result.status_code}: {result.text}")


def delete_project(collection_name: str, project_name: str, force: bool = False):
    client = get_client()
    payload = {"force": str(force).lower()}
    resp = client.delete(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}", params=payload
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
    client = get_client()
    resp = client.post(f"{SERVER_URL}/load/{collection_name}/{project_name}")
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
    client = get_client()
    try:
        resp = client.post(f"{SERVER_URL}/drop/{collection_name}/{project_name}")
    except requests.exceptions.ConnectionError:
        # The server is probably down, so this operation doesn't really matter
        return {}
    if resp.status_code == 200:
        return resp.json()
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def path_exists(collection_name: str, project_name: str, project_path: str):
    client = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/exists", params=params
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
    client = get_client()
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
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/files", params=params
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
    client = get_client()
    params = {
        "project_path": project_path,
        "real_path": folder_path,
        "type": "folder",
        "recursive": str(recursive).lower(),
    }
    resp = client.post(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/files", params=params
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
    client = get_client()
    params = {"show_hidden": str(show_hidden).lower()}
    if project_path:
        params["project_path"] = project_path

    resp = client.get(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/list", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def get_file(collection_name: str, project_name: str, project_path: str):
    client = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/files", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    else:
        raise NotFound(f"{resp.json()}")


def generate_path(collection_name: str, project_name: str, project_path: str):
    client = get_client()
    params = {"project_path": project_path}
    resp = client.get(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/generate",
        params=params,
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def remove_file(collection_name: str, project_name: str, project_path: str):
    client = get_client()
    params = {"project_path": project_path}
    resp = client.delete(
        f"{SERVER_URL}/projects/{collection_name}/{project_name}/files", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def export_tree(collection_name: str, project_name: str, output_path: Path):
    client = get_client()
    params = {"output_path": str(output_path)}
    resp = client.get(
        f"{SERVER_URL}/export/{collection_name}/{project_name}", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")


def import_tree(collection_name: str, project_name: str, input_path: Path):
    client = get_client()
    params = {"input_path": str(input_path)}
    resp = client.get(
        f"{SERVER_URL}/import/{collection_name}/{project_name}", params=params
    )
    if resp.status_code == 200:
        return resp.json()
    elif resp.status_code == 404:
        raise NotFound(f"{resp.json()}")
    else:
        raise GodataClientError(f"{resp.status_code}: {resp.text}")
