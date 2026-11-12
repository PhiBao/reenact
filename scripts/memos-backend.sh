#!/usr/bin/env bash
# (Re)start a disposable Memos backend of a given release and create the test user.
#   scripts/memos-backend.sh 0.26.2 [host-port]
# The web UI under test is served separately (vite dev server proxying to this backend),
# so the UI source can be changed while the backend stays the release the bug was filed on.
set -u
TAG=${1:-0.26.2}
PORT=${2:-8081}
NAME=memos-reenact-$PORT
docker rm -f "$NAME" >/dev/null 2>&1
docker run -d --name "$NAME" -p "$PORT:5230" "ghcr.io/usememos/memos:$TAG" >/dev/null || exit 1
for _ in $(seq 1 90); do curl -fs -o /dev/null "http://localhost:$PORT/" && break; sleep 1; done
# 0.26 to 0.31 take a flat body on /api/v1/users; 26.x moved REST to /api/users.
for path in api/v1/users api/users; do
  curl -s -X POST "http://localhost:$PORT/$path" -H 'content-type: application/json' \
    -d '{"username":"admin","password":"Passw0rd!"}' >/dev/null
done
