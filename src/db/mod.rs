pub mod mssql;
pub mod postgres;
pub mod mysql;
pub mod sqlite;

use anyhow::Result;
use sqlx::Row;
use crate::models::{DbDriver, QueryExecuteResponse, SchemaNode};

pub async fn get_databases(driver: &DbDriver, connection_string: &str) -> Result<Vec<String>> {
    match driver {
        DbDriver::Mssql => mssql::get_databases(connection_string).await,
        DbDriver::Postgres => {
            let pool = sqlx::PgPool::connect(connection_string).await?;
            let rows = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname")
                .fetch_all(&pool)
                .await?;
            Ok(rows.into_iter().map(|r| r.get(0)).collect())
        }
        DbDriver::Mysql => {
            let pool = sqlx::MySqlPool::connect(connection_string).await?;
            let rows = sqlx::query("SHOW DATABASES").fetch_all(&pool).await?;
            Ok(rows.into_iter().map(|r| r.get(0)).collect())
        }
        DbDriver::Sqlite => Ok(vec!["main".to_string()]),
    }
}

pub async fn execute_query(driver: &DbDriver, connection_string: &str, database: Option<&str>, query: &str) -> QueryExecuteResponse {
    match driver {
        DbDriver::Mssql => mssql::execute_query(connection_string, database, query).await,
        DbDriver::Postgres => postgres::execute_query(connection_string, query).await,
        DbDriver::Mysql => mysql::execute_query(connection_string, query).await,
        DbDriver::Sqlite => sqlite::execute_query(connection_string, query).await,
    }
}

pub async fn get_tree_root(driver: &DbDriver, connection_string: &str, database: Option<&str>, all_databases: bool) -> Result<Vec<SchemaNode>> {
    match driver {
        DbDriver::Mssql => mssql::get_tree_root(connection_string, database, all_databases).await,
        DbDriver::Postgres => postgres::get_tree_root(connection_string).await,
        DbDriver::Mysql => mysql::get_tree_root(connection_string).await,
        DbDriver::Sqlite => sqlite::get_tree_root(connection_string).await,
    }
}

pub async fn get_children(driver: &DbDriver, connection_string: &str, node_type: &str, parent_id: &str) -> Result<Vec<SchemaNode>> {
    match driver {
        DbDriver::Mssql => mssql::get_children(connection_string, node_type, parent_id).await,
        DbDriver::Postgres => postgres::get_children(connection_string, node_type, parent_id).await,
        DbDriver::Mysql => mysql::get_children(connection_string, node_type, parent_id).await,
        DbDriver::Sqlite => sqlite::get_children(connection_string, node_type, parent_id).await,
    }
}

pub async fn get_definition(driver: &DbDriver, connection_string: &str, node_type: &str, object_id: &str) -> Result<String> {
    match driver {
        DbDriver::Mssql => mssql::get_definition(connection_string, node_type, object_id).await,
        DbDriver::Postgres => postgres::get_definition(connection_string, node_type, object_id).await,
        DbDriver::Mysql => mysql::get_definition(connection_string, node_type, object_id).await,
        DbDriver::Sqlite => sqlite::get_definition(connection_string, node_type, object_id).await,
    }
}
