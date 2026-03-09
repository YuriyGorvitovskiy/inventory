# Domain Model

## Global Entity Identity
Every entity is identified globally with four segments:
- tenant
- owner service
- entity type
- local entity id (`int64`)

Canonical string format:
- `tenant.owner_service.entity_type.id`
- Example: `tenant-local.inventory-core.item.42`

Relational partitioning model:
- Tenant by database
- Service by schema
- Type by table
- Id by table primary key (`BIGINT`)

Identifier exposure policy:
- Product APIs primarily expose table-local numeric `id`.
- Tenant is resolved from authenticated session context, not request body/query.
- Service is encoded in API gateway route and forwarded to service boundary.
- Composite ID is used mainly for logging, tracing, and event payloads.

## Core Entities (Shared Household + PLM Path)

### Item
Represents a uniquely tracked thing.
- id
- sku_code (optional now, useful later)
- name
- description
- category_id
- default_location_id
- unit_of_measure
- quantity_on_hand
- reorder_threshold
- lifecycle_state
- created_at
- updated_at

### Category
Logical grouping.
- id
- name
- parent_id (optional)

### Location
Physical (or later virtual) storage place.
- id
- name
- parent_id (for nested locations)
- location_type

### InventoryEvent
Immutable event log for stock and lifecycle changes.
- id
- item_id
- event_type (ADD, CONSUME, ADJUST, MOVE, PURCHASE, DISPOSE)
- quantity_delta
- from_location_id
- to_location_id
- note
- event_at
- actor

### Attachment
Receipts, manuals, warranty docs, photos.
- id
- item_id
- file_name
- file_url
- attachment_type

## PLM-Ready Extensions

### PartRevision
- part_number
- revision_code
- status
- effective_from
- effective_to

### ItemRelation
For BOM-like links.
- parent_item_id
- child_item_id
- quantity
- relation_type

### ChangeOrder
- number
- title
- description
- status
- requested_by
- approved_by
- created_at
- approved_at

## Modeling Rules
- Use immutable event records for inventory movement and corrections.
- Prefer soft lifecycle transitions over hard delete.
- Keep identifiers stable and human-readable where possible (`sku_code`, `part_number`).
