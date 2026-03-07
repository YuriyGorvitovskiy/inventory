# Dynamic Entity Model Specification

## 1. Goal
Provide a runtime-extensible domain model where every entity is stored in one table row and relationships are entities too.
Support tenant-specific model and logic customization through a separate Customization Service.

## 2. Core Tables (Conceptual)
- `entities`
- `entity_types`
- `entity_fields`
- `entity_logic_bindings`
- `entity_events`
- `tenant_type_overlays`
- `tenant_logic_overlays`
- `entity_associations`

## 3. Universal Entity Shape
```json
{
  "id": "uuid",
  "tenant_id": "uuid",
  "type": "item",
  "schema_version": 3,
  "data": {},
  "status": "active",
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

Notes:
- `data` is JSONB and validated against current runtime type definition.
- Strong indexes are still required for high-use fields via generated/functional indexes.
- `tenant_id` is mandatory in all customization and lookup paths.

## 4. Relationship-As-Entity Pattern
Many-to-many links are represented as first-class entities.

Example relation entity (`item_in_location`):
```json
{
  "type": "item_in_location",
  "data": {
    "from_entity_id": "item-uuid",
    "to_entity_id": "location-uuid",
    "quantity": 2,
    "unit": "pack"
  }
}
```

Benefits:
- Uniform querying model.
- Auditable lifecycle for relationships.
- Extra attributes on relations without join-table redesign.

## 5. Runtime Model Representation
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

## 6. Dynamic Custom Logic
Runtime logic modules can be attached to entity lifecycle events.

Hooks:
- `before_create`
- `after_create`
- `before_update`
- `after_update`
- `on_event`

Execution model:
- Logic module version pinned per entity type/version.
- Activation and rollback by metadata change.
- Deterministic execution and strict timeout.
- Tenant override logic is resolved by precedence:
  1. OOTB hard invariants
  2. Tenant override hooks
  3. OOTB default hooks

## 7. Customization Service Boundary
- OOTB services own core entity definitions and lifecycle invariants.
- Customization Service owns tenant overlays, custom associations, and tenant logic bindings.
- External/custom modules can define new entity types and associate them to OOTB entities.
- Cross-boundary writes should use published APIs and events, not DB-level coupling.

Example association entity:
```json
{
  "type": "custom_association",
  "tenant_id": "tenant-uuid",
  "data": {
    "source_entity_id": "custom-entity-uuid",
    "target_entity_id": "ootb-entity-uuid",
    "association_type": "supplier_offer_for_item",
    "attributes": {"price": 4.99, "currency": "USD"}
  }
}
```

## 8. Query and Reporting Strategy
Universal table simplifies consistency but can hurt analytics if unmanaged.

Recommended pattern:
- Keep transactional writes in `entities`.
- Build service-specific read projections/materialized views.
- Index common keys extracted from JSONB.
- Push analytical workloads to derived tables where needed.

## 9. Migration Strategy
- Type evolution uses versioned definitions.
- Existing entities remain readable under previous versions.
- Optional background migration upgrades old `data` payloads.
- Tenant overlays are versioned independently from OOTB type versions.

## 10. Risks and Guardrails
Risks:
- Type/validation drift.
- JSONB query complexity.
- Runtime logic instability.
- Tenant customization leakage across tenants.

Guardrails:
- Strict schema registry and compatibility checks.
- CI validation for logic packages.
- Contract tests for type versions.
- Controlled rollout and rollback procedures.
- Tenant-scoped query filters and policy enforcement in every service.
