#!/bin/bash
godata_server &
poetry run pytest
pkill godata_server
