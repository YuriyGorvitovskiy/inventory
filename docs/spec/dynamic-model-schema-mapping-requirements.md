# Dynamic Model, Schema, and Mapping Requirements

## 1. Purpose
Define drill-down requirements for:
- dynamic model definition,
- schema generation from model,
- deterministic bidirectional mapping between model names and DB names,
- index generation strategy (including default behavior for references).

This document refines `docs/spec/persistence-requirements-overview.md` FR-7 and related requirements.

## 2. Model Definition Requirements

### DMR-1 Model Structure
- A model SHALL contain a list of **entity types**.
- Each entity type SHALL have a unique name within the model scope.
- Each entity type SHALL include a list of field descriptions.

### DMR-2 Field Definition
- Each field SHALL have a name unique within its entity type.
- Each field SHALL declare exactly one primitive from the allowed set:
  - `boolean`
  - `int64`
  - `float64`
  - `reference_int64`
  - `string_850`
  - `text`

### DMR-3 Reserved Identity Field
- Every entity type SHALL contain a predefined reserved field `id`.
- `id` primitive SHALL be `reference_int64`.
- `id` SHALL be mapped to table primary key semantics.
- `id` SHALL always have a primary key index.

### DMR-4 Optional Index Directives
- Model definitions SHALL support explicit index directives for:
  - a single field, or
  - a group of fields (composite index).
- Reference fields (`reference_int64`) SHALL be indexed by default, even if no explicit index directive is defined.

## 3. Primitive-to-Schema Mapping Requirements

### DSR-1 Canonical SQL Type Mapping
The mapping builder SHALL deterministically map primitives as follows:
- `boolean` -> `BOOLEAN`
- `int64` -> `BIGINT`
- `float64` -> `DOUBLE PRECISION`
- `reference_int64` -> `BIGINT`
- `string_850` -> `VARCHAR(850)`
- `text` -> `TEXT`

### DSR-2 Nullability and Defaults
- Nullability/default policies SHALL be explicitly represented in model metadata or convention rules.
- The mapping builder SHALL produce deterministic column nullability/default clauses from those rules.

### DSR-3 Reserved and Protected Names
- The mapping builder SHALL reject entity/field names that collide with reserved system fields beyond allowed reserved set.
- The mapping builder SHALL prevent ambiguous generated DB identifiers.

## 4. Deterministic Naming and Bidirectional Mapping

### DMRM-1 DB Identifier Normalization
- Entity type names SHALL map to table names in lowercase `snake_case`.
- Field names SHALL map to column names in lowercase `snake_case`.
- Index names SHALL map to deterministic lowercase `snake_case` forms.

### DMRM-2 Stability Guarantee
- Given the same model version and mapping rules, generated table/column/index names SHALL be identical.
- Name generation SHALL be collision-safe (e.g., suffix strategy) and deterministic.

### DMRM-3 Bidirectional Mapping
- The system SHALL maintain bidirectional mapping metadata:
  - model entity type <-> DB table
  - model field <-> DB column
  - model index definition <-> DB index name
- Mapping metadata SHALL be queryable by runtime services.

### DMRM-4 Dynamic Dictionaries
- The system SHALL support optional dynamic dictionaries for explicit aliasing/overrides between model names and DB identifiers.
- Dictionary usage SHALL be versioned and audited.
- Where dictionary entries exist, they SHALL override default normalization while preserving uniqueness constraints.

## 5. Schema Generation Workflow Requirements

### SG-1 Mapping Builder Inputs
- The mapping builder SHALL accept:
  - model definition,
  - current schema/mapping state,
  - optional dictionary overrides.

### SG-2 Generated Outputs
- The mapping builder SHALL generate:
  - target schema specification (tables/columns/indexes),
  - migration plan (create/alter/drop operations),
  - bidirectional mapping state updates.

### SG-3 Safety and Compatibility Checks
- Prior to applying schema changes, the builder SHALL validate:
  - naming collisions,
  - type compatibility,
  - index conflicts,
  - reserved-field integrity (especially `id`).

### SG-4 Queue Integration
- Schema/model changes produced by the mapping builder SHALL be represented as queueable model operations.
- Application order SHALL follow queue ordering guarantees defined in persistence requirements.


## 5A. Tenant Mapping Catalog Schema Requirements

### TMS-1 Tenant-Scoped DB Schemas
- Each tenant SHALL have its own DB schema for entity tables.
- Tenant schema name SHALL be derived from tenant name using the same deterministic lowercase `snake_case` normalization and identifier restrictions used for table/column/index mapping.
- Tenant schema names SHALL be unique within a database.

### TMS-2 Reserved Persistence Schema
- Persistence mapping metadata SHALL be stored in a reserved internal schema (for example `persistence_catalog`).
- This reserved schema name SHALL be blocked from tenant schema name mapping.
- Tenant-name-to-schema-name mapping SHALL reject collisions with reserved internal schema names.

### TMS-3 Shared Mapping Tables Across Tenants
- All tenant mapping metadata SHALL be stored in the same mapping tables inside the reserved persistence schema.
- Mapping rows SHALL include tenant scoping keys to isolate lookup/update per tenant.
- The mapping catalog SHALL support bidirectional lookup for all tenants without requiring per-tenant catalog tables.

### TMS-4 Required Catalog Tables (Logical)
- The mapping catalog SHALL maintain logical records for:
  - tenant schema registry,
  - entity type <-> table mapping,
  - field <-> column mapping,
  - index definition <-> index mapping,
  - naming dictionary overrides (optional),
  - mapping/model version history and activation status.

### TMS-5 Migration Ownership in Persistence Code
- Catalog schema definition and migrations SHALL be implemented in persistence code.
- Persistence startup SHALL execute catalog migration/upgrade checks before serving requests.
- Startup SHALL fail fast if required catalog version cannot be reached.

### TMS-6 Persistence Startup Entry Point
- Persistence component SHALL expose a startup entry point that performs, in order:
  1. DB connectivity validation,
  2. reserved persistence schema bootstrap/migration,
  3. tenant schema registry validation,
  4. mapping catalog readiness checks,
  5. worker/API start.
- Startup behavior SHALL be idempotent and safe under concurrent process starts.

## 6. Indexing Requirements

### IR-1 Mandatory Indexes
- Every generated entity table SHALL include:
  - primary key index on `id`.

### IR-2 Default Reference Indexes
- Every `reference_int64` field SHALL have an index by default.

### IR-3 Explicit Model Indexes
- Single-field and composite indexes declared in model SHALL be generated deterministically.
- The builder SHALL avoid duplicate indexes when default and explicit directives overlap.

### IR-4 Index Lifecycle
- Index add/alter/drop operations SHALL be migration-managed and auditable.

## 7. Concurrency and Runtime Requirements
- Mapping generation SHALL be deterministic under concurrent requests for the same model version.
- Model/schema activation SHALL use serialized queue application per partition to avoid conflicting DDL operations.
- Runtime readers SHALL have access to consistent mapping snapshots per active model version.

## 8. Acceptance Criteria for This Drill-Down Stage
- Allowed primitive set is explicitly defined.
- Reserved `id` semantics and indexing are explicitly defined.
- Default and explicit indexing rules are explicitly defined.
- Deterministic lower snake_case naming rules are defined for tables/columns/indexes.
- Bidirectional mapping and dynamic dictionary requirements are defined.
- Mapping builder input/output/validation/queue integration is defined.
- Tenant mapping catalog schema, reserved schema naming, and startup upgrade requirements are defined.
