use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbDriver {
    Mssql,
    Postgres,
    Mysql,
    Sqlite,
}

impl DbDriver {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => DbDriver::Postgres,
            "mysql" | "mariadb" => DbDriver::Mysql,
            "sqlite" | "sqlite3" => DbDriver::Sqlite,
            _ => DbDriver::Mssql,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConnectTestRequest {
    pub driver: String,
    pub connection_string: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub driver: Option<String>,
    pub connection_string: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueryResultTable {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: u64,
}

#[derive(Debug, Serialize)]
pub struct QueryExecuteResponse {
    pub success: bool,
    pub tables: Vec<QueryResultTable>,
    pub total_affected_rows: u64,
    pub execution_time_ms: u128,
    pub error: Option<String>,
    pub messages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaNode {
    pub id: String,
    pub text: String,
    pub node_type: String, // "TABLE", "VIEW", "SPROC", "COLUMN", "PARAMETER"
    pub value: String,
    pub has_children: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetTreeQuery {
    pub driver: Option<String>,
    pub connection_string: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetChildrenQuery {
    pub node_type: String,
    pub parent_id: String,
    pub driver: Option<String>,
    pub connection_string: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetDefinitionQuery {
    pub node_type: String,
    pub object_id: String,
    pub driver: Option<String>,
    pub connection_string: Option<String>,
}
