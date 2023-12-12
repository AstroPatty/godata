from urllib import parse

import aiohttp

from godata.utils import sanitize_project_path

"""
The client connects to the godata server and communicates with it on behalf of the 
project. On Mac and Linux, this communication is done via a unix socket with a 
REST-ish API. Windows is not supported at this time, but will probalby just have 
to use a TCP socket instead of a unix socket.

The client is stateless, so it's just a bunch of functions.
"""
SERVER_PATH = "/tmp/godata.sock"
SERVER_URL = f"http+unix://{parse.quote(SERVER_PATH, safe='')}"


class NotFound(Exception):
    pass


class AlreadyExists(Exception):
    pass


async def get_client():
    connection = aiohttp.UnixConnector(path=SERVER_PATH)
    session = aiohttp.ClientSession(connector=connection)
    return session


async def parse_response(resp):
    if resp.status == 200:
        return await resp.json()
    else:
        # Raise an exception
        # Get the text of the response
        raise NotFound(f"{await resp.text()}")


async def list_collections(show_hidden=False):
    session = await get_client()
    async with session as client:
        payload = parse.urlencode({"show_hidden": show_hidden}).lower()
        async with client.get(f"{SERVER_URL}/collections?{payload}") as resp:
            return await parse_response(resp)


async def list_projects(collection_name: str, show_hidden: bool = False):
    session = await get_client()
    async with session as client:
        payload = {"show_hidden": show_hidden}
        # encode the payload as a query string
        payload = parse.urlencode(payload).lower()
        async with client.get(
            f"{SERVER_URL}/projects/{collection_name}?{payload}"
        ) as resp:
            return await parse_response(resp)


async def create_project(
    collection_name: str,
    project_name: str,
    force: bool = False,
    storage_location: str = None,
):
    session = await get_client()
    async with session as client:
        args = {"force": force}
        if storage_location:
            args["storage_path"] = storage_location
        payload = parse.urlencode(args).lower()
        async with client.post(
            f"{SERVER_URL}/create/{collection_name}/{project_name}?{payload}"
        ) as resp:
            if resp.status == 201:
                return await resp.text()
            else:
                raise AlreadyExists(f"{await resp.text()}")


async def delete_project(collection_name: str, project_name: str, force: bool = False):
    session = await get_client()
    async with session as client:
        payload = parse.urlencode({"force": force}).lower()
        async with client.delete(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}?{payload}"
        ) as resp:
            return await parse_response(resp)


async def path_exists(collection_name: str, project_name: str, project_path: str):
    session = await get_client()
    params = {"project_path": project_path}
    # encode the payload as a query string
    payload = parse.urlencode(params)
    async with session as client:
        async with client.get(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/exists?{payload}"
        ) as resp:
            return await parse_response(resp)


@sanitize_project_path
async def link_file(
    collection_name: str,
    project_name: str,
    project_path: str,
    file_path: str,
    force: bool = False,
):
    session = await get_client()
    params = {
        "project_path": project_path,
        "real_path": file_path,
        "force": str(force).lower(),
    }
    # encode the payload as a query string
    payload = parse.urlencode(params)
    async with session as client:
        async with client.post(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/files?{payload}"
        ) as resp:
            if resp.status == 201:
                return await resp.json()
            else:
                raise AlreadyExists(f"{await resp.text()}")


async def link_folder(
    collection_name: str,
    project_name: str,
    project_path: str,
    folder_path: str,
    recursive: bool = False,
):
    session = await get_client()
    params = {
        "project_path": project_path,
        "real_path": folder_path,
        "type": "folder",
        "recursive": str(recursive).lower(),
    }
    # encode the payload as a query string
    payload = parse.urlencode(params)
    async with session as client:
        async with client.post(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/files?{payload}"
        ) as resp:
            if resp.status == 201:
                return await resp.text()
            else:
                raise AlreadyExists(f"{await resp.text()}")


@sanitize_project_path
async def list_project_contents(
    collection_name: str,
    project_name: str,
    project_path=None,
    show_hidden: bool = False,
):
    session = await get_client()
    params = {"show_hidden": show_hidden}
    if project_path:
        params["path"] = project_path

    # encode the payload as a query string
    payload = parse.urlencode(params).lower()
    async with session as client:
        async with client.get(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/list?{payload}"
        ) as resp:
            return await parse_response(resp)


async def get_file(collection_name: str, project_name: str, project_path: str):
    session = await get_client()
    params = {"project_path": project_path}
    # encode the payload as a query string
    payload = parse.urlencode(params)
    async with session as client:
        async with client.get(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/files?{payload}"
        ) as resp:
            if resp.status == 200:
                result = await resp.read()
                return result.decode()
            else:
                raise NotFound(f"{await resp.text()}")


async def generate_path(collection_name: str, project_name: str, project_path: str):
    session = await get_client()
    params = {"project_path": project_path}
    # encode the payload as a query string
    payload = parse.urlencode(params)
    async with session as client:
        async with client.get(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/generate?{payload}"
        ) as resp:
            if resp.status == 200:
                return await resp.text()
            else:
                raise NotFound(f"{await resp.text()}")


async def remove_file(collection_name: str, project_name: str, project_path: str):
    session = await get_client()
    params = {"project_path": project_path}
    # encode the payload as a query string
    payload = parse.urlencode(params)
    async with session as client:
        async with client.delete(
            f"{SERVER_URL}/projects/{collection_name}/{project_name}/files?{payload}"
        ) as resp:
            return await parse_response(resp)
