use crate::schema::{Column, Schema, Table};
use crate::schema::DataType::{BigInt, Boolean, VarChar64, VarChar850};

pub fn persistence_catalog() -> Schema {
    Schema::new("meta")
        .table(schemas())
        .table(tables())
        .table(columns())
        .table(primary_keys())
        .table(indexes())
        .table(index_columns())
}

fn schemas() -> Table {
    Table::new("schemas")
        .column("id", BigInt)
        .column("name", VarChar64)
        .primary_key("pk_schemas", ["id"])
        .index_unique("uq_schemas_name", ["name"])
}

fn tables() -> Table {
    Table::new("tables")
        .column("id", BigInt)
        .column("schema", BigInt)
        .column("name", VarChar64)
        .primary_key("pk_tables", ["id"])
        .index_unique("uq_tables_schema_name", ["schema", "name"])
}

fn columns() -> Table {
    Table::new("columns")
        .column("id", BigInt)
        .column("table", BigInt)
        .column("name", VarChar64)
        .column("data_type", VarChar64)
        .column_default("nullable", Boolean, false.to_string())
        .column_nullable("default_value", VarChar850)
        .column("ordinal", BigInt)
        .primary_key("pk_columns", ["id"])
        .index_unique("uq_columns_table_name", ["table", "name"])
}

fn primary_keys() -> Table {
    Table::new("primary_keys")
        .column("id", BigInt)
        .column("table", BigInt)
        .column("name", VarChar64)
        .primary_key("pk_primary_keys", ["id"])
        .index_unique("uq_primary_keys_table_id", ["table"])
}

fn indexes() -> Table {
    Table::new("indexes")
        .column("id", BigInt)
        .column("table", BigInt)
        .column("name", VarChar64)
        .column_default("unique_index", Boolean, false.to_string())
        .primary_key("pk_indexes", ["id"])
        .index_unique("uq_indexes_table_name", ["table", "name"])
}

fn index_columns() -> Table {
    Table::new("index_columns")
        .column("id", BigInt)
        .column("index", BigInt)
        .column("column", BigInt)
        .column("ordinal", BigInt)
        .primary_key("pk_index_columns", ["id"])
        .index_unique("uq_index_columns_index_ordinal", ["index", "ordinal"])
}
