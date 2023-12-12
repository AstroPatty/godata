#!/bin/bash
godata_server &
poetry run pytest -s
pkill godata_server
