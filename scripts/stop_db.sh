#!/usr/bin/env bash
set -x
set -eo pipefail

docker stop koru_test || true && docker rm koru_test || true
docker stop koru_redis_test || true && docker rm koru_redis_test || true
