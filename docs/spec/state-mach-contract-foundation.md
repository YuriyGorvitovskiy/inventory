# State Mach Phase 0 Contract Foundation

Status: Draft
Phase: 0
Audience: Application developers, runtime implementers, architecture reviewers
Primary authoring source: [docs/spec/state-mach-contract-foundation.md](/Users/yuriy/Development/inventory/docs/spec/state-mach-contract-foundation.md)

## Summary
- Define the minimal runtime contracts needed to prove the first `State Mach` slice.
- Keep one shared context DSL across queries, actions, and uniqueness scope.
- Keep entity storage simple: primitive fields only, one entity type per table, one entity instance per row.
- Defer aggregation, transport details, and migration execution details to later phases.

## 1. Purpose
This document proposes the minimal runtime contracts for **Phase 0: Contract Foundation** from [docs/plan/state-mach-implementation-breakdown.md](/Users/yuriy/Development/inventory/docs/plan/state-mach-implementation-breakdown.md).

The goal is to lock the smallest stable set of contracts that can:
- express one useful inventory slice,
- preserve `UI -> Business -> Data` separation,
- keep application-authoring ergonomic,
- stay serializable across process and service boundaries.

This draft is designed to be compatible with:
- [docs/spec/top-level-spec.md](/Users/yuriy/Development/inventory/docs/spec/top-level-spec.md)
- [docs/spec/dynamic-entity-model.md](/Users/yuriy/Development/inventory/docs/spec/dynamic-entity-model.md)
- [docs/spec/entity-definition-format.md](/Users/yuriy/Development/inventory/docs/spec/entity-definition-format.md)
- [docs/spec/persistence-requirements-overview.md](/Users/yuriy/Development/inventory/docs/spec/persistence-requirements-overview.md)
- [docs/architecture/state-mach-service-architecture.md](/Users/yuriy/Development/inventory/docs/architecture/state-mach-service-architecture.md)

## 2. Scope of This Draft
Phase 0 covers the written and serializable shapes for:
- `EntityDefinition`
- `ViewDefinition`
- `ContextQueryDefinition`
- `ActionDefinition`
- `EventEnvelope`
- `PatchEnvelope`
- `UniquenessPolicyDefinition`
- explicit `UI`, `Business`, and `Data` layer boundaries

This draft does **not** yet require:
- Kafka topic design,
- final remote transport APIs,
- final OpenAPI definitions,
- final persistence worker implementation,
- final UI widget catalog breadth,
- final query optimization strategy,
- model migration execution details.

## 3. Shared Design Rules

### 3.1 API serialization split
- Application-developer-facing model authoring APIs and files SHOULD accept TOML.
- Cross-service and cross-layer runtime interchange SHOULD use JSON.
- A contract MAY have both:
  - a TOML authoring representation,
  - a canonical JSON runtime representation.
- Every runtime contract MUST contain `kind` and `version`.

### 3.2 Identity and tenant rules
- Tenant scope comes from trusted auth/session context, not payload fields.
- Product CRUD APIs may continue using table-local numeric `id`.
- Cross-layer and cross-service envelopes use canonical composite IDs in the form `tenant.service.type.id`.
- Service boundary remains encoded by route or deployment boundary, not by mutable client input.

### 3.3 Layer execution model
- `UI` resolves views and interactions.
- `Business` evaluates actions, context, lifecycle, and business outcomes.
- `Data` owns model registry, query execution, patch application, uniqueness coordination, and storage.
- `Business` does not write storage directly.
- `UI` does not query storage directly; it uses declared context queries.
- Cross-layer communication remains contract-driven even when the runtime is co-located in one process.

### 3.4 Entity storage rule
- An entity definition contains only primitive fields plus field metadata.
- Each entity definition maps to exactly one owned database table.
- Each entity instance persists as exactly one row in that table.
- Cross-entity links are expressed through field-level reference metadata, not a separate `RelationDefinition` contract.

### 3.5 Single-entity mutation rule
To stay aligned with the persistence requirements:
- one patch envelope targets one entity row,
- one event envelope describes one business-relevant fact,
- actions may emit multiple events,
- event-to-patch routing may expand into multiple single-entity patch envelopes,
- mass-update execution is explicitly out of scope for Phase 0.

## 4. Vocabulary
- `type`: the logical entity type at the root of a named context block or reached by a context edge.
- `context`: a named, self-contained data-resolution block used by queries, actions, and uniqueness scope.
- `context path`: a navigable path inside a context graph, expressed through field nodes and edge nodes.
- `field node`: a node keyed as `"@field_name"` representing a scalar field on the current entity.
- `edge node`: a node keyed as `">field_name"` or `"<field_name"` representing navigation to a related entity.
- `forward edge`: a `">field_name"` edge that follows a reference field owned by the current entity.
- `reverse edge`: a `"<field_name"` edge that follows the inverse of a discovered reference from another entity.
- `alias`: a stable local name assigned to a field value so it can be used by `filter`, `select`, and `sorting`.
- `filter`: a boolean expression tree that narrows the candidate rows/entities inside a named context.
- `select`: an optional ordered list of aliases returned from a named context.
- `sorting`: an optional ordered list of alias tokens such as `+name` or `-created_at`.
- `pagination`: an optional context-local paging configuration.
- `keys`: the list of dot-separated field paths whose tuple must be unique in a `UniquenessPolicyDefinition`.

## 5. Primitive Field System

### 5.1 Supported primitive field types in the first iteration
The first iteration supports these primitive field types:
- `label`: short human-readable text up to 64 characters
- `string`: text up to 850 characters
- `text`: unbounded/max text
- `timestamp`: timestamp with timezone
- `integer`: signed 64-bit integer
- `float`: 64-bit floating-point number
- `boolean`: boolean

### 5.2 SQL mapping baseline
- `label` -> `VARCHAR(64)`
- `string` -> `VARCHAR(850)`
- `text` -> `TEXT`
- `timestamp` -> `TIMESTAMPTZ`
- `integer` -> `BIGINT`
- `float` -> `DOUBLE PRECISION`
- `boolean` -> `BOOLEAN`

### 5.3 Reference metadata
References are not a separate field contract or relation contract in Phase 0.

Instead, a primitive field MAY carry reference metadata such as:
- target entity type,
- target service,
- target field or target primary key policy,
- direction hints for context path navigation.

In the first iteration:
- the physical stored value remains primitive,
- the most common reference carrier will be `integer`,
- relation semantics are derived from field metadata plus the target entity definition.

## 6. Layer Boundaries

### 6.1 UI layer contract boundary
Inputs:
- authenticated request context,
- route/view parameters,
- requested `ViewDefinition` id,
- allowed interaction payload for a user action.

Outputs:
- resolved view tree,
- action invocation request,
- user-facing outcome descriptor.

UI layer is allowed to:
- select a view,
- bind params,
- request context data through declared queries,
- invoke declared actions,
- render alternate outcomes returned by `Business`.

UI layer is not allowed to:
- compose ad hoc SQL,
- mutate owned entities directly,
- embed business invariants outside declared contracts.

### 6.2 Business layer contract boundary
Inputs:
- `ActionInvocation`
- resolved context query results
- relevant model metadata
- uniqueness coordination outcomes when requested

Outputs:
- `ActionResult`
- `EventEnvelope[]`
- alternate outcomes for UI routing when needed
- `PatchEnvelope[]` directly or through event-to-patch routing

Business layer is allowed to:
- enforce business invariants,
- calculate derived values,
- decide lifecycle transitions,
- request uniqueness checks,
- emit domain events,
- choose alternate routes.

Business layer is not allowed to:
- persist tables directly,
- bypass model definitions,
- depend on UI widget implementation details.

### 6.3 Data layer contract boundary
Inputs:
- active model registry
- `PatchEnvelope`
- uniqueness reservation/check request
- context query execution request

Outputs:
- patch application result
- committed event publication handoff
- query result set
- deterministic uniqueness decision

Data layer is allowed to:
- own schema/table mapping,
- validate entity shape against active model,
- apply optimistic concurrency,
- manage uniqueness reservations,
- write owned storage,
- expose replicated read data later.

Data layer is not allowed to:
- invent business routes,
- encode UI presentation logic,
- redefine business meaning for events.

## 7. Contract Definitions

### 7.1 `EntityDefinition`
Purpose:
- declare one logical entity type and enough metadata to drive storage, validation, querying, and basic UI generation.

Minimum fields:
- `kind = "entity_definition"`
- `version`
- `service`
- `entity_type`
- `table`
- `description`
- `id_policy`
- `fields[]`
- `lifecycle`

Rules:
- fields MUST be primitive fields only,
- one entity definition maps to one physical table,
- nested object fields and embedded relation contracts are out of scope for Phase 0.

Field shape:
- `name`
- `type`
- `required`
- `default`
- `indexed`
- `description`
- `reference`
- `conflict_resolution`
- `ordinal`

Field metadata rules:
- `reference` is optional metadata, not a field type,
- `ordinal` is optional and used when ordered insert behavior is needed,
- `conflict_resolution` declares how concurrent or merged changes should be handled at field level.

Supported `conflict_resolution.mode` values for Phase 0:
- `last_change_wins`
- `increment`
- `decrement`
- `insert_before`
- `insert_after`

Ordered insert rules:
- `insert_before` and `insert_after` require an ordinal-capable field on the entity,
- that ordinal field is identified through `ordinal` metadata,
- the exact rebalance algorithm is implementation-defined in Phase 0, but the contract must declare that ordering exists.

Example runtime JSON:
```json
{
  "kind": "entity_definition",
  "version": "1.0.0",
  "service": "inventory-core",
  "entity_type": "item",
  "table": "items",
  "description": "Primary inventory record.",
  "id_policy": "implicit_int64",
  "fields": [
    {
      "name": "name",
      "type": "label",
      "required": true,
      "default": "",
      "indexed": true,
      "description": "Display name",
      "reference": null,
      "conflict_resolution": { "mode": "last_change_wins" },
      "ordinal": null
    },
    {
      "name": "category_id",
      "type": "integer",
      "required": false,
      "default": 0,
      "indexed": true,
      "description": "Referenced category primary key",
      "reference": {
        "target_service": "inventory-core",
        "target_entity_type": "category",
        "target_id_policy": "implicit_int64"
      },
      "conflict_resolution": { "mode": "last_change_wins" },
      "ordinal": null
    },
    {
      "name": "quantity",
      "type": "integer",
      "required": true,
      "default": 0,
      "indexed": false,
      "description": "Quantity on hand",
      "reference": null,
      "conflict_resolution": { "mode": "increment" },
      "ordinal": null
    }
  ],
  "lifecycle": {
    "create": true,
    "update": true,
    "delete": true,
    "soft_delete": false
  }
}
```

Example TOML authoring shape:
```toml
kind = "entity_definition"
version = "1.0.0"
service = "inventory-core"

[entity]
entity_type = "item"
table = "items"
description = "Primary inventory record."
id_policy = "implicit_int64"

[[field]]
name = "name"
type = "label"
required = true
default = ""
indexed = true
description = "Display name"

[field.conflict_resolution]
mode = "last_change_wins"

[[field]]
name = "category_id"
type = "integer"
required = false
default = 0
indexed = true
description = "Referenced category primary key"

[field.reference]
target_service = "inventory-core"
target_entity_type = "category"
target_id_policy = "implicit_int64"

[field.conflict_resolution]
mode = "last_change_wins"

[[field]]
name = "quantity"
type = "integer"
required = true
default = 0
indexed = false
description = "Quantity on hand"

[field.conflict_resolution]
mode = "increment"
```

### 7.2 `ViewDefinition`
Purpose:
- declare a server-resolved UI screen independent from a specific web or mobile renderer.

Minimum fields:
- `kind = "view_definition"`
- `version`
- `name`
- `entity_scope`
- `params[]`
- `context_queries[]`
- `layout`
- `interactions[]`

Phase 0 layout rules:
- keep layout declarative and shallow,
- use a small widget vocabulary,
- prefer page, table, form, text, and action-bar primitives.

Table column metadata SHOULD support:
- display label,
- value binding,
- editability,
- editor kind,
- formatting hint.

Initial editor kinds:
- `label`
- `string`
- `text`
- `email`
- `number`
- `integer`
- `float`
- `boolean`
- `timestamp`
- `reference_picker`

Example:
```json
{
  "kind": "view_definition",
  "version": "1.0.0",
  "name": "inventory.item.list",
  "entity_scope": "item",
  "params": [],
  "context_queries": [
    {
      "query": "inventory.items.list",
      "bind": "items"
    }
  ],
  "layout": {
    "type": "page",
    "title": "Items",
    "children": [
      {
        "type": "action_bar",
        "actions": ["inventory.item.create"]
      },
      {
        "type": "table",
        "rows": "$context.items.rows",
        "columns": [
          {
            "header": "Name",
            "value": "$row.name",
            "editable": true,
            "editor_kind": "label"
          },
          {
            "header": "Quantity",
            "value": "$row.quantity",
            "editable": true,
            "editor_kind": "integer"
          }
        ]
      }
    ]
  },
  "interactions": [
    {
      "event": "row_open",
      "route_to": "inventory.item.editor"
    }
  ]
}
```

Future extension reserved:
- query results MAY later annotate applicability of actions at table, row, or cell granularity,
- view definitions SHOULD remain able to consume such metadata without a breaking contract redesign.

### 7.3 `ContextQueryDefinition`
Purpose:
- declare read-only data requirements used by views and actions.

Minimum fields:
- `kind = "context_query_definition"`
- `version`
- `name`
- `context`
- `result_shape`
- `consistency`

Phase 0 query capabilities:
- navigate the model graph through field reference metadata,
- use one or more named contexts,
- select fields from any point along the context path,
- express complex boolean logic with `AND`, `OR`, `XOR`, and `NOT`,
- support sorting,
- support pagination.

Context model:
- `context` is a map of named context blocks,
- each named context is self-contained,
- each named context MUST declare `type`,
- each named context MAY declare:
  - `filter`
  - `select`
  - `sorting`
  - `pagination`
  - field and edge nodes,
- named contexts are independent from each other in Phase 0.

Graph model inside a named context:
- the context body contains two node kinds:
  - edge nodes keyed by `">field_name"` or `"<field_name"`,
  - field nodes keyed by `"@field_name"`,
- edge nodes use:
  - `>` for forward navigation over a local reference field,
  - `<` for reverse navigation over a discovered referencing field,
- edge nodes MAY declare:
  - `type` when the destination entity type is not obvious from model metadata,
  - nested edge nodes,
  - nested field nodes,
- field nodes MAY declare:
  - `alias` to name the field for filter, selection, and sorting,
  - no local condition in Phase 0.

Example context path expressions:
- `">category_id"`
- `"<item_id"`
- `"@name"`
- `"@quantity"`

Filter model:
- `filter` is a boolean expression tree,
- leaf predicates use one shared comparison shape,
- a leaf predicate contains:
  - `left`
  - `cmp`
  - `right`
- `left` and `right` operands MAY be:
  - an alias reference,
  - a literal value,
  - a parameter reference,
- boolean nodes use `AND`, `OR`, `XOR`, and `NOT`.

Selection model:
- `select` is optional,
- result aliases are introduced only by field-node `alias`,
- when present, `select` contains only alias names,
- `sorting` MUST reference aliases, not raw paths.

Sorting model:
- `sorting` is an ordered list of alias tokens,
- `+alias` means ascending order,
- `-alias` means descending order,
- an alias without prefix MAY be treated as ascending, but `+` or `-` is preferred for clarity.

Pagination model:
- offset/limit pagination is the minimum required support in Phase 0,
- cursor-based pagination may be added later without breaking the contract.

Example:
```json
{
  "kind": "context_query_definition",
  "version": "1.0.0",
  "name": "inventory.items.by_category",
  "context": {
    "main": {
      "type": "item",
      "@id": {
        "alias": "item_id"
      },
      "@name": {
        "alias": "item_name"
      },
      "@quantity": {
        "alias": "quantity"
      },
      ">category_id": {
        "type": "category",
        "@name": {
          "alias": "category_name"
        }
      },
      "filter": {
        "op": "AND",
        "args": [
          {
            "left": { "alias": "quantity" },
            "cmp": ">=",
            "right": { "literal": 1 }
          },
          {
            "op": "OR",
            "args": [
              {
                "left": { "alias": "category_name" },
                "cmp": "=",
                "right": { "literal": "Dairy" }
              },
              {
                "left": { "alias": "category_name" },
                "cmp": "=",
                "right": { "literal": "Bakery" }
              }
            ]
          }
        ]
      },
      "select": ["item_id", "item_name", "quantity", "category_name"],
      "sorting": ["+category_name", "+item_name"],
      "pagination": {
        "mode": "offset_limit",
        "default_limit": 50,
        "max_limit": 200
      }
    }
  },
  "result_shape": {
    "type": "rows"
  },
  "consistency": "read_committed"
}
```

### 7.4 `ActionDefinition`
Purpose:
- declare an invokable business capability.

Minimum fields:
- `kind = "action_definition"`
- `version`
- `name`
- `entity_scope`
- `params[]`
- `context`
- `uniqueness_policy`
- `logic`
- `success_outcome`
- `alternate_outcomes[]`

Phase 0 action rules:
- actions return business outcomes, not raw SQL plans,
- actions MAY declare one or more named contexts using the same context DSL as queries,
- business logic emits events,
- patch creation is explicit,
- alternate outcomes are first-class for uniqueness and validation flows.

Example:
```json
{
  "kind": "action_definition",
  "version": "1.0.0",
  "name": "inventory.item.create",
  "entity_scope": "item",
  "params": [
    { "name": "name", "type": "label", "required": true },
    { "name": "category_id", "type": "integer", "required": false },
    { "name": "quantity", "type": "integer", "required": true }
  ],
  "context": {
    "category_lookup": {
      "type": "category",
      "@id": {
        "alias": "category_id_value"
      },
      "@name": {
        "alias": "category_name"
      },
      "filter": {
        "left": { "alias": "category_id_value" },
        "cmp": "=",
        "right": { "param": "category_id" }
      },
      "select": ["category_id_value", "category_name"]
    }
  },
  "uniqueness_policy": "inventory.item.name_in_scope",
  "logic": {
    "mode": "declared_handler",
    "handler": "item.create.v1"
  },
  "success_outcome": {
    "type": "route",
    "target": "inventory.item.list"
  },
  "alternate_outcomes": [
    {
      "code": "rename_required",
      "type": "view_message"
    },
    {
      "code": "open_existing",
      "type": "route"
    }
  ]
}
```

### 7.5 `EventEnvelope`
Purpose:
- carry immutable business facts emitted by actions and committed mutations.

Minimum fields:
- `kind = "event"`
- `version`
- `event_id`
- `event_type`
- `tenant_id`
- `service`
- `entity_ref`
- `causation`
- `occurred_at`
- `payload`

Rules:
- event payload is immutable,
- event type naming should be stable and domain-oriented,
- committed data mutations SHOULD emit events after successful commit.

Example:
```json
{
  "kind": "event",
  "version": "1.0.0",
  "event_id": "evt_01HXYZ",
  "event_type": "inventory.item.created",
  "tenant_id": "tenant-local",
  "service": "inventory-core",
  "entity_ref": {
    "composite_id": "tenant-local.inventory-core.item.42",
    "entity_type": "item",
    "id": 42
  },
  "causation": {
    "action": "inventory.item.create",
    "request_id": "req_01HXYZ",
    "actor_id": "user_123"
  },
  "occurred_at": "2026-03-28T10:00:00Z",
  "payload": {
    "name": "Milk",
    "category_id": 3,
    "quantity": 2
  }
}
```

### 7.6 `PatchEnvelope`
Purpose:
- describe a deterministic data-layer mutation request derived from business events or direct business outcomes.

Minimum fields:
- `kind = "patch"`
- `version`
- `patch_id`
- `tenant_id`
- `target`
- `preconditions`
- `operations[]`
- `causation`

Phase 0 operation types:
- `create_entity`
- `set_fields`
- `delete_entity`

Rules:
- one patch envelope targets one entity row,
- optimistic concurrency preconditions are optional on create and expected on update/delete,
- per-field application semantics SHOULD respect the field's `conflict_resolution` metadata.

Example:
```json
{
  "kind": "patch",
  "version": "1.0.0",
  "patch_id": "pat_01HXYZ",
  "tenant_id": "tenant-local",
  "target": {
    "service": "inventory-core",
    "entity_type": "item",
    "table": "items",
    "id": 42
  },
  "preconditions": {
    "expected_version": 7
  },
  "operations": [
    {
      "type": "set_fields",
      "fields": {
        "quantity": 3,
        "updated_at": "2026-03-28T10:00:00Z"
      }
    }
  ],
  "causation": {
    "event_id": "evt_01HXYZ",
    "action": "inventory.item.adjust_quantity"
  }
}
```

### 7.7 `UniquenessPolicyDefinition`
Purpose:
- declare uniqueness coordination separately from storage exceptions so alternate business routing can happen before transaction failure becomes the normal control path.

Minimum fields:
- `kind = "uniqueness_policy_definition"`
- `version`
- `name`
- `entity_scope`
- `keys[]`
- `scope`
- `on_conflict`

Phase 0 conflict outcomes:
- `reject`
- `open_existing`
- `rename_required`
- `merge_candidate`

Phase 0 uniqueness key rules:
- uniqueness keys MAY reference fields on the root entity or on entities reachable through reference navigation,
- uniqueness key paths are forward-only in Phase 0,
- uniqueness evaluation MUST use declared paths rather than ad hoc storage joins.

`keys[]` rules:
- each key path identifies a field reachable from the root `entity_scope`,
- a key path MAY be local or may follow one or more forward reference hops,
- key paths are dot-separated strings,
- when a path edge is ambiguous, the scope context SHOULD include an explicit target type,
- if a path omits a target segment and multiple targets are possible, the runtime MAY treat that as a union or reject it; Phase 0 implementations SHOULD prefer explicit target segments,
- uniqueness comparison uses the tuple of resolved `keys[]` values.

Uniqueness scope model:
- uniqueness scope is defined by `scope.context`,
- `scope.context` uses the same named-context DSL as queries and actions,
- each named scope context MAY contain `filter`, `select`, `sorting`, and `pagination`, though most uniqueness scopes will only need `filter`,
- this allows uniqueness to express scoped uniqueness without overloading `keys[]`.

Example key path forms:
- `name`
- `category_id.name`
- `parent_id.category_id.name`
- `owner_id.email`

Example:
```json
{
  "kind": "uniqueness_policy_definition",
  "version": "1.0.0",
  "name": "inventory.item.name_in_scope",
  "entity_scope": "item",
  "keys": ["name", "category_id.name"],
  "scope": {
    "context": {
      "main": {
        "type": "item",
        "@name": {
          "alias": "item_name"
        },
        ">category_id": {
          "type": "category",
          "@name": {
            "alias": "category_name"
          }
        },
        "filter": {
          "left": { "alias": "category_name" },
          "cmp": "!=",
          "right": { "literal": "" }
        }
      }
    }
  },
  "on_conflict": "rename_required"
}
```

## 8. Supporting Runtime Messages

### 8.1 `ActionInvocation`
```json
{
  "action": "inventory.item.create",
  "tenant_id": "tenant-local",
  "params": {
    "name": "Milk",
    "category_id": 3,
    "quantity": 2
  },
  "request_context": {
    "request_id": "req_01HXYZ",
    "actor_id": "user_123"
  }
}
```

### 8.2 `ActionResult`
```json
{
  "status": "success",
  "outcome": {
    "type": "route",
    "target": "inventory.item.list"
  },
  "events": ["evt_01HXYZ"],
  "patches": ["pat_01HXYZ"]
}
```

## 9. End-to-End Example
The first proving example should be `create item`.

Flow:
1. `UI` resolves the item editor view.
2. User submits `inventory.item.create`.
3. `Business` validates params and evaluates `inventory.item.name_in_scope`.
4. If uniqueness passes, `Business` emits `inventory.item.created`.
5. `Data` converts the outcome into one or more `PatchEnvelope` values.
6. `Data` applies the patch to the `items` table.
7. `Data` confirms commit and publishes the committed event handoff.
8. `UI` receives the success route outcome and navigates to the item list.

Alternate route:
1. Uniqueness check returns collision.
2. `Business` returns `rename_required`.
3. `UI` re-renders the editor with a domain-specific message, without relying on database uniqueness failure as the primary control path.

## 10. Mapping to the Current Repository

### 10.1 Already aligned
- Existing TOML model files under [models/inventory-core](/Users/yuriy/Development/inventory/models/inventory-core) are a valid starting point for entity authoring and can be migrated toward the reconciled format in [docs/spec/entity-definition-format.md](/Users/yuriy/Development/inventory/docs/spec/entity-definition-format.md).
- Current identity rules in [docs/spec/dynamic-entity-model.md](/Users/yuriy/Development/inventory/docs/spec/dynamic-entity-model.md) and [docs/domain-model.md](/Users/yuriy/Development/inventory/docs/domain-model.md) match this draft.
- Current `State Mach` architecture already defines the intended three-layer split in [docs/architecture/state-mach-service-architecture.md](/Users/yuriy/Development/inventory/docs/architecture/state-mach-service-architecture.md).

### 10.2 Gaps to close in Phase 1
- Add concrete Rust runtime structs for all Phase 0 contracts.
- Refactor current CRUD handlers toward `ActionInvocation -> Business -> PatchEnvelope`.
- Introduce read-only `ContextQueryDefinition` execution instead of direct per-handler table access.
- Add committed mutation event emission.
- Add explicit uniqueness coordination before relying on DB uniqueness exceptions.
- Validate and migrate existing model files against the reconciled entity definition format.

## 11. Proposed Exit Criteria
Phase 0 can be considered complete when:
- each contract in Section 7 exists as a checked-in written spec,
- each contract has a Rust serializable/deserializable runtime shape,
- one `item` example is represented end to end using these shapes,
- the team agrees that no layer needs to violate its boundary for the first proving slice.

## 12. Recommended Next Steps
1. Accept this document as the Phase 0 contract baseline.
2. Validate [docs/spec/entity-definition-format.md](/Users/yuriy/Development/inventory/docs/spec/entity-definition-format.md) and the checked-in model files against the Phase 0 DTOs.
3. Add Rust DTOs/modules for the contracts in the co-located runtime.
4. Implement one narrow path: `inventory.item.create`.

## 13. Canonical Patterns

### 13.1 Single-Context Query
Use one named context when one root entity set is enough.

```json
{
  "kind": "context_query_definition",
  "version": "1.0.0",
  "name": "inventory.items.list",
  "context": {
    "main": {
      "type": "item",
      "@id": { "alias": "item_id" },
      "@name": { "alias": "item_name" },
      "@quantity": { "alias": "quantity" },
      "select": ["item_id", "item_name", "quantity"],
      "sorting": ["+item_name"]
    }
  },
  "result_shape": { "type": "rows" },
  "consistency": "read_committed"
}
```

### 13.2 Multi-Context Action
Use multiple named contexts when action logic needs separate lookups or validation sets.

```json
{
  "kind": "action_definition",
  "version": "1.0.0",
  "name": "inventory.item.assign_category",
  "entity_scope": "item",
  "params": [
    { "name": "item_id", "type": "integer", "required": true },
    { "name": "category_name", "type": "label", "required": true }
  ],
  "context": {
    "item_lookup": {
      "type": "item",
      "@id": { "alias": "item_id_value" },
      "@name": { "alias": "item_name" },
      "filter": {
        "left": { "alias": "item_id_value" },
        "cmp": "=",
        "right": { "param": "item_id" }
      },
      "select": ["item_id_value", "item_name"]
    },
    "category_lookup": {
      "type": "category",
      "@id": { "alias": "category_id_value" },
      "@name": { "alias": "category_name_value" },
      "filter": {
        "left": { "alias": "category_name_value" },
        "cmp": "=",
        "right": { "param": "category_name" }
      },
      "select": ["category_id_value", "category_name_value"]
    }
  },
  "uniqueness_policy": "inventory.item.name_in_scope",
  "logic": { "mode": "declared_handler", "handler": "item.assign_category.v1" },
  "success_outcome": { "type": "route", "target": "inventory.item.list" },
  "alternate_outcomes": []
}
```

### 13.3 Scoped Uniqueness Policy
Use plain forward-only `keys[]` and named scope contexts to describe where uniqueness is evaluated.

```json
{
  "kind": "uniqueness_policy_definition",
  "version": "1.0.0",
  "name": "inventory.item.name_in_category",
  "entity_scope": "item",
  "keys": ["name", "category_id.name"],
  "scope": {
    "context": {
      "main": {
        "type": "item",
        "@name": { "alias": "item_name" },
        ">category_id": {
          "type": "category",
          "@name": { "alias": "category_name" }
        },
        "filter": {
          "left": { "alias": "category_name" },
          "cmp": "!=",
          "right": { "literal": "" }
        }
      }
    }
  },
  "on_conflict": "rename_required"
}
```
