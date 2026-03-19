use crate::schema::{Column, Index, PrimaryKey, Schema, SqlStatement, Table, VectorAppend};
use im::Vector;

pub fn create_schema_statements(schema: &Schema) -> Vector<SqlStatement> {
    schema.tables.iter().fold(
        Vector::unit(create_schema_statement(schema)),
        |statements, table| {
            let statements = statements
                .append(create_table(schema, table))
                .append(add_primary_key(schema, table));

            table.indexes.iter().fold(statements, |statements, index| {
                statements.append(create_index(schema, table, index))
            })
        },
    )
}

pub fn drop_schema_statements(schema: &Schema) -> Vector<SqlStatement> {
    let statements = schema.tables.iter().rev().fold(Vector::new(), |statements, table| {
        let statements = table.indexes.iter().rev().fold(statements, |statements, index| {
            statements.append(drop_index(schema, index))
        });

        statements
            .append(drop_primary_key(schema, table))
            .append(drop_table(schema, table))
    });

    statements.append(drop_schema_statement(schema))
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
    let column_sql: Vector<String> = table.columns.iter().map(build_column_sql).collect();

    SqlStatement::new(
        format!(
            "CREATE TABLE IF NOT EXISTS {} (\n  {}\n)",
            qualify_table_name(schema, table),
            column_sql.iter().cloned().collect::<Vec<_>>().join(",\n  ")
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
            index.columns.iter().cloned().collect::<Vec<_>>().join(", ")
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
            primary_key.columns.iter().cloned().collect::<Vec<_>>().join(", ")
        ),
    )
}

fn qualify_table_name(schema: &Schema, table: &Table) -> String {
    format!("{}.{}", schema.name, table.name)
}

fn qualify_index_name(schema: &Schema, index: &Index) -> String {
    format!("{}.{}", schema.name, index.name)
}
