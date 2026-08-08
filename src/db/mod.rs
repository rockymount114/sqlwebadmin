pub mod mssql;
pub mod postgres;
pub mod mysql;
pub mod sqlite;

use anyhow::Result;
use crate::models::{DbDriver, QueryExecuteResponse, SchemaNode};

pub async fn execute_query(driver: &DbDriver, connection_string: &str, query: &str) -> QueryExecuteResponse {
    match driver {
        DbDriver::Mssql => mssql::execute_query(connection_string, query).await,
        DbDriver::Postgres => postgres::execute_query(connection_string, query).await,
        DbDriver::Mysql => mysql::execute_query(connection_string, query).await,
        DbDriver::Sqlite => sqlite::execute_query(connection_string, query).await,
    }
}

pub async fn get_tree_root(driver: &DbDriver, connection_string: &str) -> Result<Vec<SchemaNode>> {
    match driver {
        DbDriver::Mssql => mssql::get_tree_root(connection_string).await,
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
