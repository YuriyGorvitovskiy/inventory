use crate::schema::{SqlParameter, SqlStatement};
use im::Vector;
use sqlx::{postgres::PgArguments, query::Query, PgPool, Postgres};

pub struct Database<'a> {
    db: &'a PgPool,
}

impl<'a> Database<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, statement: &SqlStatement) -> Result<(), sqlx::Error> {
        bind_parameters(sqlx::query(&statement.sql), &statement.parameters)
            .execute(self.db)
            .await?;
        Ok(())
    }

    pub async fn execute_all(&self, statements: &Vector<SqlStatement>) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        for statement in statements {
            bind_parameters(sqlx::query(&statement.sql), &statement.parameters)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

fn bind_parameters<'q>(
    mut query: Query<'q, Postgres, PgArguments>,
    parameters: &'q Vector<SqlParameter>,
) -> Query<'q, Postgres, PgArguments> {
    for parameter in parameters {
        query = match parameter {
            SqlParameter::String(value) => query.bind(value),
            SqlParameter::Int64(value) => query.bind(*value),
            SqlParameter::Boolean(value) => query.bind(*value),
            SqlParameter::Timestamp(value) => query.bind(value),
        };
    }

    query
}
