#!/bin/bash --login
docker build -t godata-test:latest . && docker run -it godata-test:latest 
# Run this script from the root of the project to test the godata server and
# client. This docker daemon must be running.