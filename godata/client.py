from urllib import parse

import aiohttp

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


async def list_collections():
    session = await get_client()
    async with session as client:
        async with client.get(f"{SERVER_URL}/collections") as resp:
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
            f"{SERVER_URL}/projects/{collection_name}/{project_name}?{payload}"
        ) as resp:
            return await parse_response(resp)
