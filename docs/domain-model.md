# Domain Model

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
