#!/bin/bash --login
OPTIND=1
while getopts 'i' opt; do
  case $opt in
    i)
      CMD="/bin/bash"
      ;;
    *)
      CMD=""
      ;;
  esac
done
docker build -t godata-test:latest -f tests/Dockerfile . && docker run --env DATA_PATH=/home/data --env RUST_BACKTRACE=1 -v $GODATA_TEST_ROOT/test_io/test_data:/home/data -v $GODATA_TEST_ROOT/logs:/home/logs -it godata-test:latest $CMD
 
# Run this script from the root of the project to test the godata server and
# client. This docker daemon must be running.
# Assumes there is an environment variable $GODATA_TEST_ROOT that points to the 
# root of the test, for attachign the volume to the container.
