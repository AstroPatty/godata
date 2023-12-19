import socket

import requests
import urllib3

"""
An adapter for performing HTTP requests over a Unix domain socket. This code borrows
heavily from the Docker python client's implementation of the same thing:

https://github.com/docker/docker-py/blob/main/docker/transport/unixconn.py

There is a requests_unixsocket package, but it has not been updated in several years
and there is a breaking issue, so I included my own version.

I thought about using async HTTP requests, but I don't think it's necessary for this
project. If requests is good enough for the Docker client, it's good enough for me.
"""


class UnixHTTPConnection(urllib3.connection.HTTPConnection):
    def __init__(self, base_url, unix_socket, timeout=60):
        super().__init__("localhost", timeout=timeout)
        self.base_url = base_url
        self.unix_socket = unix_socket
        self.timeout = timeout

    def connect(self):
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(self.timeout)
        sock.connect(self.unix_socket)
        self.sock = sock


class UnixHTTPConnectionPool(urllib3.connectionpool.HTTPConnectionPool):
    def __init__(self, base_url, socket_path, timeout=60, maxsize=10):
        super().__init__("localhost", timeout=timeout, maxsize=maxsize)
        self.base_url = base_url
        self.socket_path = socket_path
        self.timeout = timeout

    def _new_conn(self):
        return UnixHTTPConnection(self.base_url, self.socket_path, self.timeout)


class UnixHTTPAdapter(requests.adapters.HTTPAdapter):
    __attrs__ = requests.adapters.HTTPAdapter.__attrs__ + [
        "pools",
        "socket_path",
        "timeout",
        "max_pool_size",
    ]

    def __init__(self, socket_url, timeout=60, **kwargs):
        socket_path = socket_url.replace("http+unix://", "")
        if not socket_path.startswith("/"):
            socket_path = f"/{socket_path}"
        self.socket_path = socket_path
        self.timeout = timeout
        self.pools = urllib3._collections.RecentlyUsedContainer(
            dispose_func=lambda p: p.close()
        )
        super().__init__(**kwargs)

    def get_connection(self, url, proxies=None):
        with self.pools.lock:
            pool = self.pools.get(url)
            if pool:
                return pool

            pool = UnixHTTPConnectionPool(url, self.socket_path)
            self.pools[url] = pool

        return pool

    def request_url(self, request, proxies):
        # The select_proxy utility in requests errors out when the provided URL
        # doesn't have a hostname, like is the case when using a UNIX socket.
        # Since proxies are an irrelevant notion in the case of UNIX sockets
        # anyway, we simply return the path URL directly.
        # See also: https://github.com/docker/docker-py/issues/811
        return request.path_url
