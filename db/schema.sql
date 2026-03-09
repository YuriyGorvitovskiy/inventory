CREATE TABLE IF NOT EXISTS items (
  id BIGSERIAL PRIMARY KEY,
  owner_service TEXT NOT NULL DEFAULT 'inventory-core',
  entity_type TEXT NOT NULL DEFAULT 'item',
  name TEXT NOT NULL,
  manufacturer TEXT,
  category TEXT,
  sku TEXT,
  quantity INTEGER NOT NULL DEFAULT 0 CHECK (quantity >= 0),
  location TEXT,
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_items_sku_not_null ON items (sku) WHERE sku IS NOT NULL;

-- Global entity identity convention:
-- tenant_id.owner_service.entity_type.id
-- Example: tenant-local.inventory-core.item.42
--
-- Relational partitioning:
-- 1) tenant_id -> database boundary
-- 2) owner_service -> schema boundary
-- 3) entity_type -> table boundary
-- 4) id -> row primary key (BIGINT)
