# State Mach Service Architecture

This document captures the current service-architecture direction for the `State Mach` framework and the first `Home Inventory` product.

## Design Intent

- `State Mach` is split into three layers with strict responsibility boundaries:
  - `UI`
  - `Business`
  - `Data`
- UI configuration processing should happen on the server side.
- Web and mobile clients should share common UI server processing and differ mainly in rendering and interaction handling.
- Small projects may run the three layers inside one service.
- Larger projects may split layers into independently deployed services.
- Cross-service synchronization should prefer the event bus over synchronous calls.

## Diagram Sources

### Core Layered Service

![State Mach layered service](rendered/state-mach-layered-service.svg)

```mermaid
flowchart TB
    subgraph Clients["Clients"]
        WEB["Web Client Renderer"]
        MOB["Mobile Client Renderer"]
    end

    subgraph UI["UI Layer"]
        UIAPI["UI Interface"]
        UIPROC["UI Composition and Mapping"]
        VIEWDEF["View Definitions"]
        WIDGETS["Framework Widget Catalog"]
    end

    subgraph BIZ["Business Layer"]
        ACT["Action Runtime"]
        LOGIC["Pure Business Logic"]
        UPOL["Uniqueness Policy"]
        TRIG["Lifecycle Triggers"]
        CTX["Context Query Processor"]
        EVENTS["Domain Event Stream"]
    end

    subgraph DATA["Data Layer"]
        PATCH["Patch Processor"]
        UCOORD["Uniqueness Coordination"]
        MODEL["Model and Relation Catalog"]
        STORE["Owned Service Storage"]
        REPL["Replicated Read Data"]
    end

    WEB --> UIAPI
    MOB --> UIAPI

    UIAPI --> UIPROC
    UIPROC --> VIEWDEF
    UIPROC --> WIDGETS
    UIPROC --> CTX
    UIPROC --> ACT
    UIPROC --> UPOL

    ACT --> LOGIC
    LOGIC --> EVENTS
    EVENTS --> TRIG
    TRIG --> ACT
    CTX --> LOGIC
    CTX --> UIPROC
    LOGIC --> UPOL
    TRIG --> UPOL

    LOGIC --> PATCH
    TRIG --> PATCH
    UPOL --> UCOORD
    PATCH --> UCOORD
    PATCH --> MODEL
    PATCH --> STORE
    STORE --> CTX
    REPL --> CTX

    classDef ui fill:#d7ecff,stroke:#3c6e91,stroke-width:2px,color:#102a43;
    classDef biz fill:#fde7c7,stroke:#b26a00,stroke-width:2px,color:#5c3b00;
    classDef data fill:#d9f2df,stroke:#2f855a,stroke-width:2px,color:#123524;
    classDef ext fill:#f3f4f6,stroke:#6b7280,stroke-width:1.5px,color:#111827;

    class WEB,MOB ext;
    class UIAPI,UIPROC,VIEWDEF,WIDGETS ui;
    class ACT,LOGIC,UPOL,TRIG,CTX,EVENTS biz;
    class PATCH,UCOORD,MODEL,STORE,REPL data;
```

### Co-Located vs Split Deployment

![State Mach deployment modes](rendered/state-mach-deployment-modes.svg)

```mermaid
flowchart LR
    subgraph Small["Small Project: Co-Located Service"]
        SUI["UI Layer"]
        SBIZ["Business Layer"]
        SDATA["Data Layer"]
        SUI --> SBIZ --> SDATA
    end

    subgraph Large["Larger Project: Split Services"]
        UIX["UI Service"]
        BIZX["Business Service"]
        DATAX["Data Service"]
        UIX --> BIZX --> DATAX
    end

    CLIENT["Client Renderers"]
    BUS["Cross-Service Event Bus"]

    CLIENT --> SUI
    CLIENT --> UIX
    BIZX <--> BUS
    DATAX <--> BUS

    classDef small fill:#eaf4ff,stroke:#4c78a8,stroke-width:2px,color:#13293d;
    classDef large fill:#fff1db,stroke:#c17c00,stroke-width:2px,color:#5c3b00;
    classDef neutral fill:#f5f5f5,stroke:#6b7280,stroke-width:1.5px,color:#111827;

    class SUI,SBIZ,SDATA small;
    class UIX,BIZX,DATAX large;
    class CLIENT,BUS neutral;
```

### Cross-Service Ownership and Event Interchange

![State Mach cross-service ownership](rendered/state-mach-cross-service-ownership.svg)

```mermaid
flowchart LR
    subgraph Inv["Inventory Service"]
        INVB["Business Layer"]
        INVU["Uniqueness Policy"]
        INVD["Owned Inventory Data"]
        INVR["Replicated External Data"]
    end

    subgraph Shop["Shopping Service"]
        SHOPB["Business Layer"]
        SHOPU["Uniqueness Policy"]
        SHOPD["Owned Shopping Data"]
        SHOPR["Replicated External Data"]
    end

    subgraph Vision["Vision Service"]
        VISB["Business Layer"]
        VISD["Owned Vision Data"]
    end

    BUS["Cross-Service Event Bus"]
    UBUS["Uniqueness Coordination Bus"]

    INVB --> BUS
    SHOPB --> BUS
    VISB --> BUS

    BUS --> INVB
    BUS --> SHOPB
    BUS --> VISB

    INVB --> INVD
    SHOPB --> SHOPD
    VISB --> VISD
    INVB --> INVU
    SHOPB --> SHOPU
    INVU <--> UBUS
    SHOPU <--> UBUS

    BUS --> INVR
    BUS --> SHOPR

    classDef svc fill:#e0f2fe,stroke:#0369a1,stroke-width:2px,color:#082f49;
    classDef data fill:#dcfce7,stroke:#15803d,stroke-width:2px,color:#052e16;
    classDef repl fill:#fef3c7,stroke:#b45309,stroke-width:2px,color:#451a03;
    classDef bus fill:#f5f3ff,stroke:#7c3aed,stroke-width:2px,color:#2e1065;
    classDef uniq fill:#ffe4e6,stroke:#be123c,stroke-width:2px,color:#4c0519;

    class INVB,SHOPB,VISB svc;
    class INVU,SHOPU uniq;
    class INVD,SHOPD,VISD data;
    class INVR,SHOPR repl;
    class BUS,UBUS bus;
```

### Uniqueness Coordination Processing

![State Mach uniqueness coordination](rendered/state-mach-uniqueness-coordination.svg)

```mermaid
flowchart LR
    ACTION["Action Runtime"]
    LOGIC["Business Logic"]
    POLICY["Uniqueness Policy"]
    PATCH["Patch Proposal"]
    UBUS["Uniqueness Coordination Bus"]
    UREG["Uniqueness Registry"]
    DECISION["Coordination Result"]
    ROUTEOK["Normal Patch Route"]
    ROUTEALT["Alternative Business Route"]
    STORE["Conflict-Free Storage"]

    ACTION --> LOGIC --> POLICY
    POLICY --> PATCH
    POLICY --> UBUS
    UBUS --> UREG
    UREG --> DECISION
    DECISION --> ROUTEOK
    DECISION --> ROUTEALT
    ROUTEOK --> STORE
    ROUTEALT --> ACTION
    PATCH --> ROUTEOK

    classDef biz fill:#fde7c7,stroke:#b26a00,stroke-width:2px,color:#5c3b00;
    classDef uniq fill:#ffe4e6,stroke:#be123c,stroke-width:2px,color:#4c0519;
    classDef data fill:#d9f2df,stroke:#2f855a,stroke-width:2px,color:#123524;

    class ACTION,LOGIC,POLICY,ROUTEALT biz;
    class UBUS,UREG,DECISION uniq;
    class PATCH,ROUTEOK,STORE data;
```

## Notes

- The `UI Layer` owns view composition, widget selection, and data-to-widget mapping, but not widget implementation.
- The `Business Layer` owns actions, lifecycle triggers, event production, pure-function logic evaluation over `params + context tree`, and uniqueness policy definition.
- The `Data Layer` owns model metadata, uniqueness coordination primitives, patch application, authoritative storage, and replicated read-side data.
- Uniqueness should not rely on transaction failures as a normal control-flow mechanism. Business logic should be able to define alternative routes for uniqueness-violation outcomes.
- The same logical architecture should work whether the layers are deployed together or separately.
