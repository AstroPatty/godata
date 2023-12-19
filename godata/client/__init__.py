from packaging import version

from . import client


def check_server():
    from .. import __version__

    server_version = version.parse(client.get_version())
    client_version = version.parse(__version__)
    if server_version.major != client_version.major:
        raise client.GodataClientError(
            f"Server version {server_version} not compatible with client version"
            f"{client_version}. Server and client must have the same major version."
        )
    elif server_version.minor < client_version.minor:
        raise client.GodataClientError(
            f"Client version cannot be newer than server version. "
            f"Server version: {server_version} < Client version: {client_version}."
        )
    else:
        return True
