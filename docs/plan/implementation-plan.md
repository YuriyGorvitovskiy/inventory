# Implementation Plan

## 1. Delivery Strategy
Three-track progression:
- Track H (Household Value): deliver day-to-day utility.
- Track P (Platform/PLM): preserve architecture that scales to Cloud PLM.
- Track M (Meta-Model): prove runtime-extensible model and dynamic logic safely.

Migration streams:
- Stream D (Model/Data Migration): business model/data/logic/UI evolution.
- Stream X (Platform Migration): infrastructure/framework/runtime/look-and-feel/internal machinery evolution.

## 2. Phases and Deliverables

### Phase 0: Foundation + Tenant-Scoped Data Topology (Week 1)
Deliverables:
- Monorepo layout for Rust microservices.
- Local k3s cluster bootstrap scripts and baseline manifests.
- Postgres + Kafka running in cluster.
- API gateway stub with health checks.
- Meta-Model service skeleton.
- Tenant DB + service schema conventions and bootstrap scripts.
- OTEL collector deployment and basic telemetry export from one sample service.

Exit criteria:
- `kubectl get pods` healthy for core infra.
- One sample Rust service reachable via gateway.
- POC CRUD over service-owned typed tables with tenant-scoped connection context.
- Baseline dashboard shows request count/latency/error for gateway + sample service.

### Phase 1: Core Inventory Vertical Slice on Typed Relational Model (Weeks 2-3)
Deliverables:
- Inventory service using typed relational tables.
- Identity service minimal auth + JWT.
- UI with login and dynamic inventory forms.
- Runtime type definitions for item/category/location/relation entities.
- Kafka event publication for stock changes.

Exit criteria:
- End-to-end: login -> add item -> consume item -> see updated quantity.
- Relationship entities correctly model item-location and category links.

### Phase 2: Customization Service + Runtime Logic Engine (Weeks 4-5)
Deliverables:
- Separate `customization-service` for tenant overlays.
- Tenant-specific model overlays for fields/views/type extensions.
- Logic runtime service (rules/hook execution) with tenant scoping.
- Logic package versioning, activation, rollback API.
- Compatibility checks for model and logic versions.
- Audit trail for model/logic changes.
- Autoscaling baseline for non-critical services (KEDA/HPA policies).
- Model/Data migration toolkit (versioned migrations, dry-run, rollback hooks).

Exit criteria:
- Tenant overlay can add field/view without changing OOTB service code.
- Activate updated tenant logic without service redeploy.
- Roll back to previous tenant logic version with no downtime.
- At least one non-critical service scales to zero and scales up successfully on demand.
- One model/data migration scenario executed end-to-end with validation report.

### Phase 3: Capture + Enrichment (Weeks 6-7)
Deliverables:
- Vision capture service (image upload + item candidate extraction).
- Web enrichment service (metadata normalization).
- UI capture flow with human confirmation.
- Event choreography between capture, enrichment, inventory.

Exit criteria:
- Upload image, receive item suggestions, confirm, entity created.

### Phase 4: Replenishment + Analytics + Assistant (Weeks 8-9)
Deliverables:
- Replenishment service (missing/perished/low-stock shopping list).
- Analytics/prediction service (simple demand forecast).
- AI assistant API service with action execution endpoints.
- UI shopping and insights pages.

Exit criteria:
- Shopping recommendations generated automatically.
- Assistant can query inventory and trigger safe actions.

### Phase 5: Hardening + Access Control Model (Week 10)
Deliverables:
- Observability stack (logs, metrics, traces).
- Backup/restore runbook.
- Advanced access control design and implementation plan.
- Security hardening for runtime model/logic changes.
- Tiered autoscaling policy tuning and cold-start optimization.
- Platform migration playbook (runtime upgrade + gateway upgrade + rollback drill).

Exit criteria:
- Disaster recovery test done.
- Security and operability checklist passed.
- Cold-start SLO validated for scale-from-zero services.
- One platform migration drill completed with measured downtime and rollback timing.

## 3. Suggested Tech Stack
- Language: Rust (`axum`, `tokio`, `serde`, `sqlx`).
- API: REST first, gRPC optional later.
- Gateway: Traefik or Envoy.
- Database: PostgreSQL 16+ with JSONB and functional indexes.
- Messaging: Kafka + schema conventions.
- Auth: minimal custom auth service first, then external IdP if needed.
- UI: React/Next.js with dynamic form renderer from model metadata.
- Runtime logic: WASM modules or DSL rules (evaluate both in Phase 2).
- Observability: OpenTelemetry SDK + OTEL Collector + Prometheus/Grafana + Tempo/Loki.
- Autoscaling: HPA for steady services, KEDA for event-driven and optional scale-to-zero.

## 4. Risks and Mitigation
- Multi-tenant DB lifecycle and operational sprawl.
  - Mitigation: provisioning automation, tenant onboarding/offboarding runbooks, backup policy by tenant tier.
- Runtime logic introduces instability.
  - Mitigation: sandbox limits, staged rollout, fast rollback.
- Cross-service coupling between OOTB and customization services.
  - Mitigation: strict ownership contracts and event-first integration.
- User-visible delay from scale-to-zero cold starts.
  - Mitigation: tiered scaling, keep critical services warm, pre-warm rules for peak windows.
- Unsafe migration coupling (business and platform changes shipped together).
  - Mitigation: separate Stream D and Stream X pipelines with independent approvals.
- AI scope creep.
  - Mitigation: define narrow assistant intents and approved actions.
