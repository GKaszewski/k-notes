#!/usr/bin/env bash
set -euo pipefail

IMAGE="${IMAGE:-registry.gabrielkaszewski.dev/k-notes:latest}"

docker buildx build --platform linux/amd64 \
  -t "$IMAGE" --push .

echo "pushed $IMAGE"
