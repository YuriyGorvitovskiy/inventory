# State Mach Implementation Breakdown

This plan turns the current `State Mach` architecture into an implementation sequence for the `inventory` workspace.

It assumes:
- `Home Inventory` is the first product on the framework.
- The first cut is a co-located service with logical `UI`, `Business`, and `Data` layers.
- Service splits come later, after the core contracts are proven inside one runtime.

## 1. Delivery Approach

Implementation should move in three nested scopes:

1. `Framework contracts first`
   - Define the minimum runtime contracts that everything else builds on.
2. `Single-service proving slice second`
   - Prove the contracts inside one service before splitting layers or services.
3. `Product value immediately after`
   - Use `Home Inventory` to validate that the framework is reducing application effort rather than adding ceremony.

## 2. Phase Breakdown

### Phase 0: Contract Foundation

Goal:
- Define the minimal contracts required for a useful `State Mach` runtime.

Deliverables:
- `EntityDefinition` contract
- `RelationDefinition` contract
- `ViewDefinition` contract
- `ContextQueryDefinition` contract
- `ActionDefinition` contract
- `Event` envelope
- `Patch` envelope
- `UniquenessPolicyDefinition`
- explicit `UI / Business / Data` layer boundaries

Exit criteria:
- Each contract exists as a written spec and serializable runtime representation.
- A single end-to-end example can be expressed without hand-waving.

### Phase 1: Co-Located Runtime Skeleton

Goal:
- Implement one co-located service that contains all three layers, but enforces them as separate modules and interfaces.

Deliverables:
- `ui` composition interface
- `business` action runtime
- `data` patch application boundary
- in-process event stream
- in-process context query execution
- model registry loading from static definitions

Exit criteria:
- One action can execute through all three layers.
- One view can be resolved through server-side UI processing.

### Phase 2: Data and Patch Core

Goal:
- Make the `Data Layer` safe, deterministic, and conflict-minimizing.

Deliverables:
- patch model and patch-application pipeline
- deterministic entity/table/column mapping
- model-driven schema generation for a small entity set
- conflict-free write strategy for owned data
- event emission from committed data mutations
- read model / projection loading path

Exit criteria:
- A declared entity model can generate and back a live table.
- Patch application updates storage and emits stable events.

### Phase 3: UI Runtime Foundation

Goal:
- Prove the server-driven UI model.

Deliverables:
- server-side `ViewDefinition` resolver
- framework widget catalog abstraction
- widget data mapping contract
- supported interaction contract
- `params + context tree` input pipeline for view rendering

Exit criteria:
- A web UI screen can be rendered from view configuration plus context data.
- No business-specific widget implementation is required for the first screens.

### Phase 4: Business Runtime Foundation

Goal:
- Prove the pure-function business execution model.

Deliverables:
- action execution pipeline
- lifecycle trigger points for `create`, `update`, `copy`, and `delete`
- action result format based on events
- event-to-patch route
- alternate route handling for non-success outcomes

Exit criteria:
- At least one action executes as:
  - input params
  - context query
  - business logic
  - emitted events
  - patch route

### Phase 5: Uniqueness Coordination

Goal:
- Implement uniqueness without making transaction failure the normal control path.

Deliverables:
- `UniquenessPolicyDefinition`
- uniqueness coordination interface
- uniqueness registry / reservation primitive
- deterministic uniqueness outcomes
- alternate business routing on uniqueness collision
- initial uniqueness coordination bus abstraction

Exit criteria:
- A uniqueness rule can be declared from the `Business Layer`.
- Parallel attempts result in deterministic coordination outcomes.
- Business logic can choose an alternative route when uniqueness is not granted.

### Phase 6: Home Inventory MVP on State Mach

Goal:
- Deliver the first useful product slice on top of the framework.

Deliverables:
- `Item` entity
- `Category` entity
- basic relation between them
- inventory list view
- create item action
- update item action
- delete item action
- one declared uniqueness rule
- one alternate route for uniqueness collision

Exit criteria:
- User can create, edit, and delete inventory items through server-driven UI.
- At least one business rule runs through the full event-and-patch pipeline.

### Phase 7: Split-Ready Interfaces

Goal:
- Preserve the option to split layers or services after the first proving slice.

Deliverables:
- externalizable event contracts
- context-query service boundary definition
- uniqueness coordination boundary definition
- patch submission boundary definition
- replicated-data contract for cross-service local reads

Exit criteria:
- The co-located service can be split conceptually without redesigning the contracts.

## 3. First MVP Slice

The first proving slice should stay narrow.

Recommended MVP:
- entities:
  - `item`
  - `category`
- relations:
  - `item.category_id`
- views:
  - item list
  - item editor
- actions:
  - create item
  - update item
  - delete item
- uniqueness:
  - unique item name within a selected scope
- alternate route:
  - collision returns a business outcome such as `merge_candidate`, `rename_required`, or `open_existing`

This is enough to prove:
- static definitions
- server-side UI resolution
- context-query-driven execution
- event output
- patch application
- uniqueness coordination

## 4. Dependency-Ordered Backlog

### Stream A: Runtime Contracts

Priority:
1. Define `Event` envelope
2. Define `Patch` envelope
3. Define `ActionDefinition`
4. Define `ContextQueryDefinition`
5. Define `ViewDefinition`
6. Define `UniquenessPolicyDefinition`

### Stream B: Data Layer

Priority:
1. Finalize model-to-schema mapping rules
2. Implement model registry loading
3. Implement patch application pipeline
4. Emit events from committed patch application
5. Implement conflict-minimizing owned-data writes
6. Implement uniqueness coordination primitives

### Stream C: Business Layer

Priority:
1. Implement action runtime contract
2. Implement context-query execution
3. Implement lifecycle trigger contract
4. Implement event-returning business execution
5. Implement alternate routing for uniqueness outcomes

### Stream D: UI Layer

Priority:
1. Define widget catalog interface
2. Define view layout contract
3. Define widget data mapping contract
4. Implement server-side view resolver
5. Implement first inventory list/editor screens

### Stream E: Home Inventory Product Slice

Priority:
1. Define `item` and `category`
2. Map first generated schema
3. Build create/update/delete actions
4. Build item list/editor views
5. Add first uniqueness rule
6. Validate alternate route behavior

## 5. Suggested Near-Term Milestones

### Milestone M1: Contracts Locked
- The runtime shapes are documented and agreed.

### Milestone M2: One Action Through Three Layers
- One action travels through `UI -> Business -> Data`.

### Milestone M3: One Real Inventory Screen
- One useful inventory screen is fully server-driven.

### Milestone M4: Uniqueness Without Transaction-Failure Control Flow
- Parallel uniqueness scenarios resolve deterministically.

### Milestone M5: Home Inventory MVP
- First practical slice is usable end to end.

## 6. Immediate Next Tasks

1. Write the explicit JSON/TOML/struct shapes for:
   - `Event`
   - `Patch`
   - `ActionDefinition`
   - `ContextQueryDefinition`
   - `ViewDefinition`
   - `UniquenessPolicyDefinition`
2. Choose the first MVP uniqueness scope for `item`.
3. Define the first item list and item editor views.
4. Implement the action runtime path inside `inventory-core`.
5. Add the first event-to-patch pipeline.
