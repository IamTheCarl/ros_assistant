#!/usr/bin/env bash

# DOCKER_HOST can be overwritten for things like non-root Docker setups.
DOCKER_HOST=${DOCKER_HOST:-"/var/run/docker.sock"}

# DOCKER_HOST often starts with `unix://` so we may need to remove that.
DOCKER_HOST_PATH=${DOCKER_HOST#unix://}

docker run \
  --rm \
  -it \
  --name builder \
  --privileged \
  -v $(pwd)/addon:/data \
  -v ${DOCKER_HOST_PATH}:/var/run/docker.sock:ro \
  ghcr.io/home-assistant/amd64-builder \
  -t /data \
  --all \
  --test \
  -i ros-assistant-{arch} \
  -d local