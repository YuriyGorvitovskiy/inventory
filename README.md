# Inventory

Household inventory manager and Cloud PLM prototype platform.

## What Is Implemented Now
- Rust workspace scaffold.
- First microservice: `inventory-core` (Axum HTTP API).
- Basic web UI at `GET /` with inline table editing and `+ Add row`.
- PostgreSQL persistence for inventory items.
- Health endpoints:
  - `GET /health`
  - `GET /ready`
- Item CRUD API:
  - `GET /api/items`
  - `POST /api/items`
  - `PUT /api/items/{id}`
  - `DELETE /api/items/{id}`
- Model inspection API:
  - `GET /api/model`
- API identifiers are numeric per-entity-table (`int64` / `BIGINT`).
- Tenant context is resolved from authenticated session/token claims.
- Service routing is encoded in gateway paths (example: `/inventory/...`).
- Composite global IDs are primarily for logs, tracing, and events.
- Dockerfile for `inventory-core`.
- One-line Docker pipeline script (`scripts/docker-pipeline.sh`).
- Kubernetes manifests (`kustomize`) for local cluster deployment.

## Project Layout
- `services/inventory-core` - first Rust service
- `models/inventory-core` - interpretive entity model files (one TOML per entity type)
- `deploy/k8s/base` - Kubernetes manifests
- `db/schema.sql` - initial DB schema draft
- `docs/` - architecture and planning docs

## Prerequisites
- Rust toolchain (`rustup`, `cargo`)
- Docker
- Kubernetes (`k3s` + `kubectl`)
- Optional: `kustomize` (or `kubectl apply -k`)

## 1. One-Line Docker Deploy (Recommended)
```bash
cd /Users/yuriy/Development/inventory
./scripts/docker-pipeline.sh
```

Or via make:
```bash
make docker-deploy
```

This will:
- build image `inventory-core:dev`
- ensure Postgres container `inventory-postgres` is running
- replace existing `inventory-core` container
- start container on `localhost:8080`
- wait until `/health` is ready

Open UI:
```bash
open http://localhost:8080/
```

Stop container:
```bash
make docker-stop
```

Stop app and database containers:
```bash
make docker-stop-all
```

## 2. Build and Run Locally (Rust)
```bash
cd /Users/yuriy/Development/inventory
cargo run -p inventory-core
```

Optional tenant context:
```bash
TENANT_ID=tenant-acme cargo run -p inventory-core
```

Test endpoints:
```bash
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

Open UI:
```bash
open http://localhost:8080/
```

## 3. Build Container Manually
```bash
cd /Users/yuriy/Development/inventory
docker build -t inventory-core:dev -f services/inventory-core/Dockerfile .
```

## 4. Deploy to Kubernetes (Optional)
```bash
cd /Users/yuriy/Development/inventory
kubectl apply -k deploy/k8s/base
kubectl -n inventory get pods
kubectl -n inventory get svc
```

Port-forward and test:
```bash
kubectl -n inventory port-forward svc/inventory-core 8080:80
curl http://localhost:8080/health
```

## Helper Commands
```bash
make run
make build
make docker-build
make docker-deploy
make docker-stop
make docker-stop-all
make k8s-apply
make k8s-delete
```

## Planning Documents
- `docs/spec/top-level-spec.md`
- `docs/spec/dynamic-entity-model.md`
- `docs/spec/entity-definition-format.md`
- `docs/spec/state-mach-contract-foundation.md`
- `docs/plan/implementation-plan.md`
- `docs/plan/work-breakdown.md`
- `docs/plan/persistence-plan.md`
- `docs/spec/persistence-requirements-overview.md`
- `docs/spec/dynamic-model-schema-mapping-requirements.md`
