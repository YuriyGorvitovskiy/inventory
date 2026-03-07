# Inventory (MVP)

A simple inventory web app you can run on your MacBook:
- Rust app server (`axum` + `sqlx`)
- PostgreSQL database (single `items` table)
- Browser UI with CRUD (list, add, edit, delete)

## Data model
The app stores items in one table (`items`) with fields:
- `name`
- `manufacturer`
- `category`
- `sku`
- `quantity`
- `location`
- `description`

Schema is in `db/schema.sql`.

## Run locally
### 1) Start PostgreSQL
```bash
docker compose up -d db
```

### 2) Configure environment
```bash
cp .env.example .env
export $(cat .env | xargs)
```

### 3) Run the Rust server
```bash
cargo run
```

Open: <http://localhost:3000>

## API
- `GET /api/items` — list items
- `POST /api/items` — create item
- `PUT /api/items/:id` — update item
- `DELETE /api/items/:id` — delete item

## Notes
- On startup, the server executes `db/schema.sql` to ensure the table exists.
- `quantity` cannot be negative.
- `sku` is unique when provided.
