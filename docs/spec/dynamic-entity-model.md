# Dynamic Entity Model Specification

## 1. Goal
Provide a runtime-extensible model while preserving clear ownership and strong tenant isolation in relational storage.
Entity metadata is interpreted from model files at service startup/runtime.

Model file format reference:
- `docs/spec/entity-definition-format.md`
- one file per entity type under `models/<service>/*.entity.toml`

## 2. Persistence Topology
- Tenant isolation: one database per tenant.
- Service ownership: one schema per service inside the tenant database.
- Entity typing: one table per entity type inside a service schema.
- Entity row identity: `BIGINT` primary key per table.

Example:
- Database: `tenant_acme`
- Schema: `inventory_core`
- Table: `items`
- Row id: `42`

## 3. Identity Model
Canonical global identity segments:
- tenant
- owner service
- entity type
- table-local id (`int64`)

Canonical string form:
- `tenant.owner_service.entity_type.id`
- Example: `tenant-acme.inventory-core.item.42`

## 4. API and Routing Model
- API gateway route encodes service boundary (example: `/inventory/items/{id}`).
- Tenant is resolved from authenticated session/token claims.
- Services do not trust tenant input from request payload/query for authorization scope.
- Standard product APIs use numeric `id` values.

## 5. Where Composite IDs Are Used
Composite global IDs are primarily for:
- logs
- traces
- events
- cross-service audit records

They are not required as primary user-facing identifiers in normal CRUD APIs.

## 6. Runtime Model Representation
Meta-Model Service exposes:
- Type definitions
- Field definitions
- Constraints and validation rules
- Relation semantics
- UI hints

Example representation:
```json
{
  "entity_type": "item",
  "version": 3,
  "fields": [
    {"key": "name", "type": "string", "required": true},
    {"key": "expiry_date", "type": "date", "required": false}
  ],
  "relations": [
    {"type": "item_in_location", "target": "location"}
  ]
}
```

## 7. Customization Service Boundary
- OOTB services own core entity definitions and lifecycle invariants.
- Customization Service owns tenant overlays, custom associations, and tenant logic bindings.
- External/custom modules can define new entity types and associate them to OOTB entities.
- Cross-boundary writes should use published APIs and events, not DB-level coupling.

## 8. Query and Reporting Strategy
- Keep transactional writes in service-owned relational tables.
- Build service-specific read projections/materialized views where needed.
- Add targeted indexes per query shape.
- Push analytical workloads to derived/reporting tables.

## 9. Migration Strategy
- Type evolution uses versioned definitions.
- Existing entities remain readable under previous versions.
- Optional background migration upgrades payload shape/data meaning.
- Tenant overlays are versioned independently from OOTB type versions.

## 10. Risks and Guardrails
Risks:
- Type/validation drift across services.
- Cross-service ownership leakage.
- Tenant isolation mistakes in routing/auth.
- Runtime logic instability.

Guardrails:
- Strict ownership contracts by service schema boundary.
- Tenant context enforcement from auth/session claims.
- CI validation for logic packages.
- Contract tests for type versions.
- Controlled rollout and rollback procedures.
