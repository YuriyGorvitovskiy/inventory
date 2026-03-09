use sqlx::PgPool;

pub async fn ensure_schema(db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_items (
          id BIGSERIAL PRIMARY KEY,
          owner_service TEXT NOT NULL DEFAULT 'inventory-core',
          entity_type TEXT NOT NULL DEFAULT 'item',
          name TEXT NOT NULL,
          category TEXT NOT NULL,
          quantity BIGINT NOT NULL CHECK (quantity >= 0),
          created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
          updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE inventory_items
        ADD COLUMN IF NOT EXISTS owner_service TEXT NOT NULL DEFAULT 'inventory-core'
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE inventory_items
        ADD COLUMN IF NOT EXISTS entity_type TEXT NOT NULL DEFAULT 'item'
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}
