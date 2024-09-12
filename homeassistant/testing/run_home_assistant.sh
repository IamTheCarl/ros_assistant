#!/usr/bin/env bash

# DOCKER_HOST can be overwritten for things like non-root Docker setups.
DOCKER_HOST=${DOCKER_HOST:-"/var/run/docker.sock"}

# DOCKER_HOST often starts with `unix://` so we may need to remove that.
DOCKER_HOST_PATH=${DOCKER_HOST#unix://}

# mkdir -p testing/config/custom_components

docker run --rm \
  --name homeassistant \
  --privileged \
  -v ./testing/config:/config \
  -v ./homeassistant/integration:/config/custom_components/ros_assistant \
  --network=host \
  ghcr.io/home-assistant/home-assistant:stable
