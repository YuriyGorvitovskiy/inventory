use crate::schema::{Column, Index, PrimaryKey, Schema, SqlStatement, Table};

pub fn create_schema_statements(schema: &Schema) -> Vec<SqlStatement> {
    let mut statements = vec![create_schema_statement(schema)];

    for table in &schema.tables {
        statements.push(create_table(schema, table));
        statements.push(add_primary_key(schema, table));

        for index in &table.indexes {
            statements.push(create_index(schema, table, index));
        }
    }

    statements
}

pub fn drop_schema_statements(schema: &Schema) -> Vec<SqlStatement> {
    let mut statements = Vec::new();

    for table in schema.tables.iter().rev() {
        for index in table.indexes.iter().rev() {
            statements.push(drop_index(schema, index));
        }

        statements.push(drop_primary_key(schema, table));
        statements.push(drop_table(schema, table));
    }
    statements.push(drop_schema_statement(schema));

    statements
}

pub fn create_schema_statement(schema: &Schema) -> SqlStatement {
    SqlStatement::new(
        format!("CREATE SCHEMA IF NOT EXISTS {}", schema.name),
    )
}

pub fn drop_schema_statement(schema: &Schema) -> SqlStatement {
    SqlStatement::new(
        format!("DROP SCHEMA IF EXISTS {}", schema.name),
    )
}

pub fn create_table(schema: &Schema, table: &Table) -> SqlStatement {
    let column_sql: Vec<String> = table.columns.iter().map(build_column_sql).collect();

    SqlStatement::new(
        format!(
            "CREATE TABLE IF NOT EXISTS {} (\n  {}\n)",
            qualify_table_name(schema, table),
            column_sql.join(",\n  ")
        ),
    )
}

pub fn drop_table(schema: &Schema, table: &Table) -> SqlStatement {
    SqlStatement::new(
        format!("DROP TABLE IF EXISTS {}", qualify_table_name(schema, table)),
    )
}

pub fn add_primary_key(schema: &Schema, table: &Table) -> SqlStatement {
    create_primary_key(schema, table, &table.primary_key)
}

pub fn drop_primary_key(schema: &Schema, table: &Table) -> SqlStatement {
    SqlStatement::new(
        format!(
            "ALTER TABLE {} DROP CONSTRAINT IF EXISTS {}",
            qualify_table_name(schema, table),
            table.primary_key.name
        ),
    )
}

pub fn create_index(schema: &Schema, table: &Table, index: &Index) -> SqlStatement {
    let unique = if index.unique { "UNIQUE " } else { "" };
    SqlStatement::new(
        format!(
            "CREATE {}INDEX IF NOT EXISTS {} ON {} ({})",
            unique,
            qualify_index_name(schema, index),
            qualify_table_name(schema, table),
            index.columns.join(", ")
        ),
    )
}

pub fn drop_index(schema: &Schema, index: &Index) -> SqlStatement {
    SqlStatement::new(
        format!("DROP INDEX IF EXISTS {}", qualify_index_name(schema, index)),
    )
}

fn build_column_sql(column: &Column) -> String {
    let mut parts = vec![column.name.clone(), column.data_type.sql().to_string()];
    if !column.nullable {
        parts.push("NOT NULL".to_string());
    }
    if let Some(default) = &column.default {
        parts.push(format!("DEFAULT {default}"));
    }
    parts.join(" ")
}

fn create_primary_key(schema: &Schema, table: &Table, primary_key: &PrimaryKey) -> SqlStatement {
    SqlStatement::new(
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} PRIMARY KEY ({})",
            qualify_table_name(schema, table),
            primary_key.name,
            primary_key.columns.join(", ")
        ),
    )
}

fn qualify_table_name(schema: &Schema, table: &Table) -> String {
    format!("{}.{}", schema.name, table.name)
}

fn qualify_index_name(schema: &Schema, index: &Index) -> String {
    format!("{}.{}", schema.name, index.name)
}
