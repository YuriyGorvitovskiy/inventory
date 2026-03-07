# Roadmap

## Track A: Household Inventory (Immediate Value)

### Milestone A1: Core Data and CRUD
- Item catalog
- Categories
- Locations
- Quantity tracking
- Purchase details

### Milestone A2: Daily Operations
- Consume/restock flows
- Low-stock notifications
- Expiry tracking (optional)
- Barcode/QR support (optional)

### Milestone A3: Reporting
- Value by location/category
- Monthly consumption summary
- Missing/unknown location report

## Track B: Cloud PLM Prototype (Strategic)

### Milestone B1: Part/Revision Model
- Part master
- Revision history
- Effective dates/status

### Milestone B2: Structure and Change
- BOM-like parent-child links
- Change request / change order objects
- Approval/status flow

### Milestone B3: Multi-tenant Cloud Readiness
- Account/workspace boundary
- Role-based access controls
- API versioning and audit compliance

## Design Principle
Build household features directly on top of shared entities and event history so they become PLM building blocks, not throwaway code.
