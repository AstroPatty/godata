import multiprocessing as mp


def setup_module(module):
    # Make sure the server is running
    from godata.server import stop

    try:
        stop()
    except RuntimeError:
        # If the server is already not running, that's fine.

        pass


def test_many_start():
    from godata.server import start

    # max number of processes available
    n = mp.cpu_count()
    if n < 2:
        raise RuntimeError("This test requires at least 2 CPUs")
    # run the command in n threads, and collect the return values
    with mp.Pool(n) as pool:
        results = pool.map(start, range(n))
    assert results[0] and not any(results[1:])
