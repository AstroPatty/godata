#!/bin/bash
poetry run pytest
pkill godata_server
