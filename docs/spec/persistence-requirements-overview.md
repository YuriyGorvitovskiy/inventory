# Persistence Requirements Overview

## 1. Objective
Define a persistence architecture where:
- each operation targets exactly one entity row,
- entities remain independent and conflict-free under concurrent updates,
- reads support both single-entity and cross-table graph-style reporting,
- schema and model evolve dynamically with safe rollout,
- writes and model changes are accepted as queued persistent payloads.

This document is a requirements baseline for deeper drill-down docs.

## 2. Core Principles
1. **One entity = one table row**
   - Every domain entity instance is persisted as a single row in its owning table.
   - Relationship concepts are represented explicitly (join/relation tables), not embedded blobs.
2. **Single-entity operation atomicity**
   - A single operation mutates one entity row only.
   - A payload may contain multiple operations; each operation remains single-entity scoped.
3. **Queue-first persistence**
   - Persistent payloads are accepted, stored durably, and processed asynchronously by workers.
4. **Model + data changes share execution rail**
   - Schema/model-change commands are queued and sequenced with data operations.
5. **Concurrency by design**
   - Multi-threaded load/search/report/update workers are first-class requirements.

## 3. Functional Requirements

### FR-1 Entity Persistence (Single-Row)
- The system SHALL persist each entity instance as one row in one owned table.
- The system SHALL assign stable entity identity and version metadata per row.
- The system SHALL reject any single operation that attempts to mutate multiple entity rows.

### FR-2 Payload Contract and Queueing
- The system SHALL accept a **persistent payload** consisting of an ordered list of operations.
- The system SHALL store payloads durably before acknowledging acceptance.
- Each operation in a payload SHALL target one entity only.
- The system SHALL support payload types:
  - `data_operation` (create/update/delete/upsert single entity),
  - `model_operation` (add/alter/deprecate entity definitions),
  - `mass_update_operation` (declarative fan-out into many single-entity operations).
- The system SHALL support idempotency keys at payload and operation levels.

### FR-3 Ordered and Safe Execution
- The queue processor SHALL preserve ordering guarantees within a defined partition scope (tenant + entity type or stronger key).
- The processor SHALL expose operation states (`accepted`, `queued`, `running`, `succeeded`, `failed`, `dead-lettered`).
- Failed operations SHALL not corrupt queue continuity; retry and dead-letter policies are required.

### FR-4 Conflict-Free Independent Persistence
- The system SHALL provide optimistic concurrency control (row version/check token) for entity updates.
- Conflicts SHALL be detected deterministically and surfaced as explicit conflict outcomes.
- Independent entities SHALL be updatable in parallel without global locking.

### FR-5 Concurrent Processing and Reads
- The system SHALL support multi-threaded workers for update/load/search/report flows.
- The system SHALL define isolation behavior for readers against in-flight writes.
- The system SHALL support back-pressure controls for queue consumers and report jobs.

### FR-6 Cross-Table Search and Graph-Like Reports
- The system SHALL support cross-table search across owned entity tables.
- The system SHALL support graph-like report definitions (nodes = entities, edges = relationships, projections/aggregations).
- Report definitions SHALL be versioned and executable against current model versions.

### FR-7 Dynamic Schema and Model Evolution
- Model changes SHALL be expressed as queueable operations.
- Model operations SHALL run through compatibility checks before activation.
- The system SHALL support rolling migration strategies that avoid blocking normal single-entity operations.
- The system SHALL keep audit history for model version activation/rollback.
- The system SHALL maintain mapping metadata in a reserved persistence schema with tenant-scoped records in shared catalog tables.
- The system SHALL run persistence catalog upgrades during startup before queue/API processing is enabled.

### FR-8 Mass Update Support
- The system SHALL support mass update requests (predicate + mutation specification).
- A mass update SHALL be internally expanded into traceable single-entity operations.
- Mass updates SHALL provide progress reporting, pause/cancel controls, and partial-failure accounting.

### FR-9 Auditability and Observability
- The system SHALL emit auditable records for payload acceptance, operation execution, retries, failures, and schema changes.
- The system SHALL expose metrics for queue depth, operation latency, retry rate, conflict rate, and report runtime.

## 4. Non-Functional Requirements
- **Durability:** accepted payloads survive process/node restarts.
- **Consistency:** deterministic ordering and conflict semantics in defined partition scopes.
- **Scalability:** horizontal worker scaling for high-throughput updates and report workloads.
- **Isolation:** tenant-safe execution boundaries for data and model operations.
- **Performance:** bounded latency targets for payload acceptance and eventual execution.
- **Operability:** replay/recovery tooling, dead-letter inspection, and migration observability.

## 5. Logical Capability Model
- **Ingress API**: validates payloads, writes to queue store, returns tracking IDs.
- **Queue Store**: durable ordered log + status index.
- **Execution Workers**: apply single-entity operations and model operations.
- **Conflict Manager**: OCC/version checks and retry policy integration.
- **Model Runtime**: validates and activates dynamic schema/model versions.
- **Search/Report Engine**: executes cross-table queries and graph-like report definitions.
- **Audit/Telemetry**: immutable audit trail and metrics/traces/logs.

## 6. Open Design Questions for Drill-Down
1. Partition strategy for strict ordering vs throughput (`tenant`, `tenant+entity_type`, or finer).
2. OCC token format (integer version, timestamp/version pair, hash, etc.).
3. Mass update execution semantics (snapshot set vs rolling set).
4. Compatibility matrix between model versions and in-flight data operations.
5. Report-definition DSL/format for graph-like queries.
6. SLA/SLO targets for queue lag and report completion.
7. Reserved internal schema name set and tenant-name collision policy details.

## 6A. Drill-Down Artifacts
- Dynamic model/schema/mapping drill-down: `docs/spec/dynamic-model-schema-mapping-requirements.md`.

## 7. Acceptance Criteria for This Requirements Stage
- Requirements explicitly state single-entity operation boundaries.
- Queueing model includes both data and model changes.
- Concurrency/conflict requirements are defined.
- Cross-table search/report and dynamic schema requirements are included.
- Mass update behavior is defined as expansion into single-entity operations.
