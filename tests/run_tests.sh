#!/bin/bash
godata_server &
poetry run python tests/test_projects.py
pkill godata_server
