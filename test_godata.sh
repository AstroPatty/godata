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
# Run a redis server in a networked container
docker network create godata-net

# check if redis is already running
docker ps | grep redis
if [ $? -eq 0 ]; then
  echo "Redis is already running"
else
  echo "Starting redis"
  docker run --network godata-net -d redis
fi

# Get the IP address of the redis server
REDIS_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $(docker ps -q --filter ancestor=redis))

docker build -t godata-test:latest .\
&& \
docker run \
  --network godata-net \
  --env DATA_PATH=/home/data \
  --env RUST_BACKTRACE=1 \
  --env REDIS_HOST=$REDIS_IP \
  -v $GODATA_TEST_ROOT/test_io/test_data:/home/data \
  -it godata-test:latest $CMD
# Run this script from the root of the project to test the godata server and
# client. This docker daemon must be running.
# Assumes there is an environment variable $GODATA_TEST_ROOT that points to the 
# root of the test, for attachign the volume to the container.
