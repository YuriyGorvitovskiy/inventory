PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  parent_id INTEGER,
  FOREIGN KEY (parent_id) REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS locations (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  parent_id INTEGER,
  location_type TEXT DEFAULT 'physical',
  FOREIGN KEY (parent_id) REFERENCES locations(id)
);

CREATE TABLE IF NOT EXISTS items (
  id INTEGER PRIMARY KEY,
  sku_code TEXT UNIQUE,
  name TEXT NOT NULL,
  description TEXT,
  category_id INTEGER,
  default_location_id INTEGER,
  unit_of_measure TEXT NOT NULL DEFAULT 'unit',
  quantity_on_hand REAL NOT NULL DEFAULT 0,
  reorder_threshold REAL NOT NULL DEFAULT 0,
  lifecycle_state TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (category_id) REFERENCES categories(id),
  FOREIGN KEY (default_location_id) REFERENCES locations(id)
);

CREATE TABLE IF NOT EXISTS inventory_events (
  id INTEGER PRIMARY KEY,
  item_id INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  quantity_delta REAL NOT NULL,
  from_location_id INTEGER,
  to_location_id INTEGER,
  note TEXT,
  actor TEXT,
  event_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (item_id) REFERENCES items(id),
  FOREIGN KEY (from_location_id) REFERENCES locations(id),
  FOREIGN KEY (to_location_id) REFERENCES locations(id)
);

CREATE TABLE IF NOT EXISTS attachments (
  id INTEGER PRIMARY KEY,
  item_id INTEGER NOT NULL,
  file_name TEXT NOT NULL,
  file_url TEXT NOT NULL,
  attachment_type TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (item_id) REFERENCES items(id)
);

-- PLM-ready extension tables

CREATE TABLE IF NOT EXISTS part_revisions (
  id INTEGER PRIMARY KEY,
  item_id INTEGER NOT NULL,
  part_number TEXT NOT NULL,
  revision_code TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft',
  effective_from TEXT,
  effective_to TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(part_number, revision_code),
  FOREIGN KEY (item_id) REFERENCES items(id)
);

CREATE TABLE IF NOT EXISTS item_relations (
  id INTEGER PRIMARY KEY,
  parent_item_id INTEGER NOT NULL,
  child_item_id INTEGER NOT NULL,
  quantity REAL NOT NULL DEFAULT 1,
  relation_type TEXT NOT NULL DEFAULT 'component',
  FOREIGN KEY (parent_item_id) REFERENCES items(id),
  FOREIGN KEY (child_item_id) REFERENCES items(id)
);

CREATE TABLE IF NOT EXISTS change_orders (
  id INTEGER PRIMARY KEY,
  number TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'open',
  requested_by TEXT,
  approved_by TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  approved_at TEXT
);

CREATE TABLE IF NOT EXISTS change_order_items (
  id INTEGER PRIMARY KEY,
  change_order_id INTEGER NOT NULL,
  item_id INTEGER NOT NULL,
  target_revision_id INTEGER,
  action_type TEXT NOT NULL,
  FOREIGN KEY (change_order_id) REFERENCES change_orders(id),
  FOREIGN KEY (item_id) REFERENCES items(id),
  FOREIGN KEY (target_revision_id) REFERENCES part_revisions(id)
);

CREATE INDEX IF NOT EXISTS idx_items_category ON items(category_id);
CREATE INDEX IF NOT EXISTS idx_items_location ON items(default_location_id);
CREATE INDEX IF NOT EXISTS idx_events_item ON inventory_events(item_id);
CREATE INDEX IF NOT EXISTS idx_events_time ON inventory_events(event_at);
CREATE INDEX IF NOT EXISTS idx_part_revisions_item ON part_revisions(item_id);
CREATE INDEX IF NOT EXISTS idx_rel_parent ON item_relations(parent_item_id);
