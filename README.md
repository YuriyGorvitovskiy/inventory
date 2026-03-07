# Inventory

Household inventory manager and Cloud PLM prototype platform.

## Planning Documents
- Top-level specification: `docs/spec/top-level-spec.md`
- Dynamic entity model: `docs/spec/dynamic-entity-model.md`
- Implementation plan: `docs/plan/implementation-plan.md`
- Work breakdown: `docs/plan/work-breakdown.md`

## Current Direction
- Rust microservices on Kubernetes.
- PostgreSQL as system of record.
- Kafka for async service communication.
- Runtime-extensible universal entity model.
- Runtime-updatable custom logic with versioning and rollback.
- OpenTelemetry-based observability (metrics, traces, logs).
- Tiered autoscaling with selective scale-to-zero and fast scale-up.
- Two migration streams: model/data migration and platform migration.
- Access control model implemented in later hardening phase.

## Next Step
Review and approve these docs, then we convert Phase 0 + First Sprint into concrete tickets with acceptance criteria and estimates.
