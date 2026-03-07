#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="inventory-core"
IMAGE_TAG="dev"
CONTAINER_NAME="inventory-core"
DB_CONTAINER_NAME="inventory-postgres"
DB_IMAGE="postgres:16-alpine"
DB_VOLUME="inventory-postgres-data"
DOCKER_NETWORK="inventory-net"
DB_NAME="inventory"
DB_USER="inventory"
DB_PASSWORD="inventory"
HOST_PORT="8080"
CONTAINER_PORT="8080"
HEALTH_URL="http://localhost:${HOST_PORT}/health"
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_CONTAINER_NAME}:5432/${DB_NAME}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/5] Ensuring docker network ${DOCKER_NETWORK}..."
if ! docker network inspect "${DOCKER_NETWORK}" >/dev/null 2>&1; then
  docker network create "${DOCKER_NETWORK}" >/dev/null
fi

echo "[2/5] Ensuring Postgres container ${DB_CONTAINER_NAME}..."
if docker ps --format '{{.Names}}' | grep -qx "${DB_CONTAINER_NAME}"; then
  echo "Postgres is already running."
elif docker ps -a --format '{{.Names}}' | grep -qx "${DB_CONTAINER_NAME}"; then
  docker start "${DB_CONTAINER_NAME}" >/dev/null
else
  docker volume create "${DB_VOLUME}" >/dev/null
  docker run -d \
    --name "${DB_CONTAINER_NAME}" \
    --network "${DOCKER_NETWORK}" \
    -e POSTGRES_DB="${DB_NAME}" \
    -e POSTGRES_USER="${DB_USER}" \
    -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
    -v "${DB_VOLUME}:/var/lib/postgresql/data" \
    -p 5432:5432 \
    "${DB_IMAGE}" >/dev/null
fi

echo "Waiting for Postgres readiness..."
for i in {1..30}; do
  if docker exec "${DB_CONTAINER_NAME}" pg_isready -U "${DB_USER}" -d "${DB_NAME}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "[3/5] Building image ${IMAGE_NAME}:${IMAGE_TAG}..."
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" -f services/inventory-core/Dockerfile .

if docker ps -a --format '{{.Names}}' | grep -qx "${CONTAINER_NAME}"; then
  echo "[4/5] Removing previous container ${CONTAINER_NAME}..."
  docker rm -f "${CONTAINER_NAME}" >/dev/null
else
  echo "[4/5] No previous container found."
fi

echo "[5/5] Starting container ${CONTAINER_NAME} on :${HOST_PORT}..."
docker run -d \
  --name "${CONTAINER_NAME}" \
  --network "${DOCKER_NETWORK}" \
  -p "${HOST_PORT}:${CONTAINER_PORT}" \
  -e RUST_LOG=info \
  -e DATABASE_URL="${DATABASE_URL}" \
  "${IMAGE_NAME}:${IMAGE_TAG}" >/dev/null

echo "Waiting for health endpoint ${HEALTH_URL}..."
for i in {1..30}; do
  if curl -fsS "${HEALTH_URL}" >/dev/null 2>&1; then
    echo "Service is healthy."
    echo "UI: ${HEALTH_URL%/health}/"
    echo "Postgres persistence volume: ${DB_VOLUME}"
    exit 0
  fi
  sleep 1
done

echo "Service did not become healthy in time. Recent logs:"
docker logs --tail 200 "${CONTAINER_NAME}" || true
exit 1
