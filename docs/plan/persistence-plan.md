# Persistence Planning (Inventory Core)

## Goal
Move from a single-table bootstrap persistence layer to a durable, migration-driven, tenant-safe persistence foundation that supports current CRUD and future dynamic entity growth.

## Current Baseline
- `inventory-core` persists items in Postgres table `inventory_items` via inline SQL in HTTP handlers.
- Schema bootstrap is runtime-driven (`ensure_schema`) and only guarantees table + two compatibility columns.
- SQL uses direct `sqlx::query` / `query_as` calls inside endpoint handlers.


## Requirements Baseline
This planning track is derived from `docs/spec/persistence-requirements-overview.md`, which captures the required operational model for:
- single-entity row operations,
- queue-based execution for both data and model changes,
- concurrent conflict-safe updates,
- cross-table search and graph-like reporting,
- dynamic schema evolution and mass updates.
- dynamic model/schema mapping contracts and deterministic DB naming.

## Scope for This Planning Track
1. Make schema evolution explicit and repeatable.
2. Isolate data access from transport handlers.
3. Add persistence-focused tests (unit + integration).
4. Prepare table model for future entity expansion (category/location/relations).

## Non-Goals (for this track)
- No cross-service event choreography.
- No multi-database provisioning automation.
- No full dynamic-schema runtime yet (only preparatory structure).

## Proposed Workstreams

### W1. Schema and Migration Discipline
- Adopt SQLx migration files under `services/inventory-core/migrations/`.
- Move bootstrap SQL from runtime `CREATE TABLE IF NOT EXISTS` to versioned migrations.
- Add explicit `down` strategy notes (or forward-only policy + rollback runbook decision).
- Keep startup guard: run migrations on boot in dev/local environments.

**Deliverables**
- Initial migration chain for existing `inventory_items` schema.
- Migration policy documented in repo (naming, ordering, rollback expectations).

### W2. Persistence Layer Extraction
- Introduce a repository module (for example `src/repository/items.rs`) to own SQL.
- Keep handlers focused on HTTP concerns (validation, status mapping, serialization).
- Define clear persistence errors mapped to API-layer errors.

**Deliverables**
- `list/create/update/delete` item DB operations moved out of handlers.
- Consistent error translation path for SQL/state errors.

### W3. Data Model Hardening
- Add constraints/indexes that match API expectations:
  - index on `(owner_service, entity_type)` for service-level scans,
  - optional uniqueness strategy for future business keys,
  - explicit checks/defaults for quantity and timestamps.
- Normalize column intent for tenant-aware identity evolution.

**Deliverables**
- Migration(s) adding indexes/constraints.
- Updated schema notes in `db/schema.sql` (as design reference).


### W5. Mapping Catalog Schema and Startup Lifecycle
- Define and implement reserved persistence catalog schema for mapping metadata.
- Store all tenant mappings in shared catalog tables with tenant-scoped keys.
- Add persistence startup entry point that runs catalog migrations/upgrades before serving traffic.
- Enforce tenant schema naming restrictions and reserved-schema collision prevention.

**Deliverables**
- Catalog schema DDL + migration chain in persistence codebase.
- Startup sequence with fail-fast migration/version checks.
- Validation rules for tenant schema names and reserved-name rejection.

### W4. Testing and Verification
- Add integration tests for repository CRUD against Postgres test DB.
- Add migration smoke test in CI or make target (`sqlx migrate run` + sanity queries).
- Validate handler behavior still returns expected status codes for not found/validation/database errors.

**Deliverables**
- Repeatable local test commands for persistence validation.
- Minimum regression suite around CRUD + migration application.

## Execution Sequence (Suggested)
1. W1 migrations foundation.
2. W2 repository extraction.
3. W3 indexing/constraint hardening.
4. W5 mapping catalog schema and startup lifecycle.
5. W4 tests and CI checks.

## Risks and Mitigations
- **Risk:** Migration drift between runtime SQL and checked-in schema.
  - **Mitigation:** remove runtime schema mutation except controlled migration runner.
- **Risk:** Refactor introduces handler regressions.
  - **Mitigation:** lock behavior with endpoint tests before/after refactor.
- **Risk:** Future dynamic entities conflict with current table assumptions.
  - **Mitigation:** codify current table as `item` baseline and document extension path.

## Definition of Done (Planning Track Complete)
- Persistence implementation backlog is decomposed into migration/repository/testing tasks.
- First executable migration chain exists and can initialize a clean DB.
- Item CRUD no longer embeds SQL in handlers.
- Persistence checks run via one documented command sequence.
- Persistence startup upgrades mapping catalog schema successfully before serving requests.
