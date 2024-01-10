#!/bin/bash
poetry run pytest -W ignore::ResourceWarning
pkill godata_server
