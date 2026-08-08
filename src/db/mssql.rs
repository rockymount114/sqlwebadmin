use anyhow::{anyhow, Result};
use serde_json::Value;
use std::time::Instant;
use tiberius::{Client, Config, Row};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use crate::models::{QueryResultTable, QueryExecuteResponse, SchemaNode};

pub type TiberiusClient = Client<Compat<TcpStream>>;

pub async fn connect(connection_string: &str) -> Result<TiberiusClient> {
    let config = Config::from_ado_string(connection_string)
        .map_err(|e| anyhow!("Invalid MSSQL connection string: {}", e))?;
    
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|e| anyhow!("Failed to connect to MSSQL host {}: {}", config.get_addr(), e))?;
    
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp.compat_write())
        .await
        .map_err(|e| anyhow!("MSSQL handshake failed: {}", e))?;

    Ok(client)
}

pub async fn execute_query(connection_string: &str, query: &str) -> QueryExecuteResponse {
    let start = Instant::now();
    
    let mut client = match connect(connection_string).await {
        Ok(c) => c,
        Err(e) => {
            return QueryExecuteResponse {
                success: false,
                tables: vec![],
                total_affected_rows: 0,
                execution_time_ms: start.elapsed().as_millis(),
                error: Some(e.to_string()),
                messages: vec![],
            };
        }
    };

    let stream = match client.simple_query(query).await {
        Ok(s) => s,
        Err(e) => {
            return QueryExecuteResponse {
                success: false,
                tables: vec![],
                total_affected_rows: 0,
                execution_time_ms: start.elapsed().as_millis(),
                error: Some(e.to_string()),
                messages: vec![],
            };
        }
    };

    let mut tables = Vec::new();
    let mut total_affected = 0u64;

    let result_sets = match stream.into_results().await {
        Ok(rs) => rs,
        Err(e) => {
            return QueryExecuteResponse {
                success: false,
                tables: vec![],
                total_affected_rows: 0,
                execution_time_ms: start.elapsed().as_millis(),
                error: Some(e.to_string()),
                messages: vec![],
            };
        }
    };

    for rs in result_sets {
        let mut col_names = Vec::new();
        if !rs.is_empty() {
            let first_row = &rs[0];
            for col in first_row.columns() {
                col_names.push(col.name().to_string());
            }
        }

        let mut rows_data = Vec::new();
        let row_count = rs.len() as u64;

        for row in rs {
            if col_names.is_empty() {
                for col in row.columns() {
                    col_names.push(col.name().to_string());
                }
            }

            let mut row_values = Vec::new();
            for i in 0..row.columns().len() {
                let val = extract_column_value(&row, i);
                row_values.push(val);
            }
            rows_data.push(row_values);
        }

        total_affected += row_count;

        tables.push(QueryResultTable {
            columns: col_names,
            rows: rows_data,
            affected_rows: row_count,
        });
    }

    QueryExecuteResponse {
        success: true,
        tables,
        total_affected_rows: total_affected,
        execution_time_ms: start.elapsed().as_millis(),
        error: None,
        messages: vec![format!("Query executed successfully in {} ms.", start.elapsed().as_millis())],
    }
}

fn extract_column_value(row: &Row, idx: usize) -> Value {
    // Attempt standard type conversions safely via try_get
    if let Ok(Some(s)) = row.try_get::<&str, _>(idx) {
        return Value::from(s);
    }
    if let Ok(Some(i)) = row.try_get::<i32, _>(idx) {
        return Value::from(i);
    }
    if let Ok(Some(i)) = row.try_get::<i64, _>(idx) {
        return Value::from(i);
    }
    if let Ok(Some(i)) = row.try_get::<i16, _>(idx) {
        return Value::from(i);
    }
    if let Ok(Some(i)) = row.try_get::<u8, _>(idx) {
        return Value::from(i);
    }
    if let Ok(Some(f)) = row.try_get::<f64, _>(idx) {
        return Value::from(f);
    }
    if let Ok(Some(f)) = row.try_get::<f32, _>(idx) {
        return Value::from(f);
    }
    if let Ok(Some(b)) = row.try_get::<bool, _>(idx) {
        return Value::from(b);
    }
    if let Ok(Some(dt)) = row.try_get::<chrono::NaiveDateTime, _>(idx) {
        return Value::from(dt.to_string());
    }
    if let Ok(Some(d)) = row.try_get::<chrono::NaiveDate, _>(idx) {
        return Value::from(d.to_string());
    }
    if let Ok(Some(bytes)) = row.try_get::<&[u8], _>(idx) {
        let hex_str = format!("0x{}", bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>());
        return Value::from(hex_str);
    }

    Value::Null
}

pub async fn get_tree_root(_connection_string: &str) -> Result<Vec<SchemaNode>> {
    Ok(vec![
        SchemaNode {
            id: "TABLE".to_string(),
            text: "Tables".to_string(),
            node_type: "TABLE".to_string(),
            value: "TABLE".to_string(),
            has_children: true,
        },
        SchemaNode {
            id: "VIEW".to_string(),
            text: "Views".to_string(),
            node_type: "VIEW".to_string(),
            value: "VIEW".to_string(),
            has_children: true,
        },
        SchemaNode {
            id: "SPROC".to_string(),
            text: "Stored Procedures".to_string(),
            node_type: "SPROC".to_string(),
            value: "SPROC".to_string(),
            has_children: true,
        },
    ])
}

pub async fn get_children(connection_string: &str, node_type: &str, parent_id: &str) -> Result<Vec<SchemaNode>> {
    let mut client = connect(connection_string).await?;
    let mut nodes = Vec::new();

    match node_type {
        "TABLE" => {
            let sql = "SELECT 'TABLE.' + SCHEMA_NAME(schema_id) + '.' + name AS Id, SCHEMA_NAME(schema_id) + '.' + name AS Name FROM sys.tables ORDER BY SCHEMA_NAME(schema_id), name";
            let stream = client.simple_query(sql).await?;
            let result_sets = stream.into_results().await?;
            for rs in result_sets {
                for row in rs {
                    let id: Option<&str> = row.try_get(0).ok().flatten();
                    let name: Option<&str> = row.try_get(1).ok().flatten();
                    if let (Some(id_val), Some(name_val)) = (id, name) {
                        nodes.push(SchemaNode {
                            id: id_val.to_string(),
                            text: name_val.to_string(),
                            node_type: "TABLE_ITEM".to_string(),
                            value: id_val.to_string(),
                            has_children: true,
                        });
                    }
                }
            }
        }
        "VIEW" => {
            let sql = "SELECT 'VIEW.' + SCHEMA_NAME(schema_id) + '.' + name AS Id, SCHEMA_NAME(schema_id) + '.' + name AS Name FROM sys.views ORDER BY SCHEMA_NAME(schema_id), name";
            let stream = client.simple_query(sql).await?;
            let result_sets = stream.into_results().await?;
            for rs in result_sets {
                for row in rs {
                    let id: Option<&str> = row.try_get(0).ok().flatten();
                    let name: Option<&str> = row.try_get(1).ok().flatten();
                    if let (Some(id_val), Some(name_val)) = (id, name) {
                        nodes.push(SchemaNode {
                            id: id_val.to_string(),
                            text: name_val.to_string(),
                            node_type: "VIEW_ITEM".to_string(),
                            value: id_val.to_string(),
                            has_children: true,
                        });
                    }
                }
            }
        }
        "SPROC" => {
            let sql = "SELECT 'SPROC.' + SCHEMA_NAME(schema_id) + '.' + name AS Id, SCHEMA_NAME(schema_id) + '.' + name AS Name FROM sys.procedures ORDER BY SCHEMA_NAME(schema_id), name";
            let stream = client.simple_query(sql).await?;
            let result_sets = stream.into_results().await?;
            for rs in result_sets {
                for row in rs {
                    let id: Option<&str> = row.try_get(0).ok().flatten();
                    let name: Option<&str> = row.try_get(1).ok().flatten();
                    if let (Some(id_val), Some(name_val)) = (id, name) {
                        nodes.push(SchemaNode {
                            id: id_val.to_string(),
                            text: name_val.to_string(),
                            node_type: "SPROC_ITEM".to_string(),
                            value: id_val.to_string(),
                            has_children: true,
                        });
                    }
                }
            }
        }
        "TABLE_ITEM" | "VIEW_ITEM" => {
            let clean_id = parent_id.trim_start_matches("TABLE.").trim_start_matches("VIEW.");
            let sql = format!(
                "SELECT 'COLUMN.' + '{}' + '.' + c.name AS Id, c.name + ' (' + t.name + CASE WHEN t.name IN ('time', 'datetime2', 'datetimeoffset', 'varbinary', 'varchar', 'binary', 'char', 'nvarchar', 'nchar') THEN '(' + CASE WHEN c.max_length = -1 THEN 'max' ELSE CAST(c.max_length AS nvarchar(50)) END + ')' ELSE '' END + ')' AS Name FROM sys.columns AS c JOIN sys.types AS t ON c.system_type_id = t.system_type_id AND c.user_type_id = t.user_type_id WHERE object_id = OBJECT_ID('{}') ORDER BY column_id",
                clean_id, clean_id
            );
            let stream = client.simple_query(&sql).await?;
            let result_sets = stream.into_results().await?;
            for rs in result_sets {
                for row in rs {
                    let id: Option<&str> = row.try_get(0).ok().flatten();
                    let name: Option<&str> = row.try_get(1).ok().flatten();
                    if let (Some(id_val), Some(name_val)) = (id, name) {
                        nodes.push(SchemaNode {
                            id: id_val.to_string(),
                            text: name_val.to_string(),
                            node_type: "COLUMN".to_string(),
                            value: id_val.to_string(),
                            has_children: false,
                        });
                    }
                }
            }
        }
        "SPROC_ITEM" => {
            let clean_id = parent_id.trim_start_matches("SPROC.");
            let sql = format!(
                "SELECT 'PARAMETER.' + '{}' + '.' + p.name AS Id, p.name + ' (' + t.name + CASE WHEN t.name IN ('time', 'datetime2', 'datetimeoffset', 'varbinary', 'varchar', 'binary', 'char', 'nvarchar', 'nchar') THEN '(' + CASE WHEN p.max_length = -1 THEN 'max' ELSE CAST(p.max_length AS nvarchar(50)) END + ')' ELSE '' END + ')' AS Name FROM sys.parameters AS p JOIN sys.types AS t ON p.system_type_id = t.system_type_id AND p.user_type_id = t.user_type_id WHERE p.object_id = OBJECT_ID('{}') ORDER BY p.parameter_id",
                clean_id, clean_id
            );
            let stream = client.simple_query(&sql).await?;
            let result_sets = stream.into_results().await?;
            for rs in result_sets {
                for row in rs {
                    let id: Option<&str> = row.try_get(0).ok().flatten();
                    let name: Option<&str> = row.try_get(1).ok().flatten();
                    if let (Some(id_val), Some(name_val)) = (id, name) {
                        nodes.push(SchemaNode {
                            id: id_val.to_string(),
                            text: name_val.to_string(),
                            node_type: "PARAMETER".to_string(),
                            value: id_val.to_string(),
                            has_children: false,
                        });
                    }
                }
            }
        }
        _ => {}
    }

    Ok(nodes)
}

pub async fn get_definition(connection_string: &str, node_type: &str, object_id: &str) -> Result<String> {
    let mut client = connect(connection_string).await?;
    let clean_id = object_id
        .trim_start_matches("TABLE.")
        .trim_start_matches("VIEW.")
        .trim_start_matches("SPROC.");

    if node_type == "TABLE_ITEM" || node_type == "TABLE" {
        let col_sql = format!(
            "SELECT QUOTENAME(c.Name) AS Id, QUOTENAME(c.Name) AS Name FROM sys.columns AS c JOIN sys.types AS t ON c.system_type_id = t.system_type_id AND c.user_type_id = t.user_type_id WHERE c.object_id = OBJECT_ID('{}') ORDER BY column_id",
            clean_id
        );
        let stream = client.simple_query(&col_sql).await?;
        let result_sets = stream.into_results().await?;
        let mut cols = Vec::new();
        for rs in result_sets {
            for row in rs {
                if let Ok(Some(col_name)) = row.try_get::<&str, _>(0) {
                    cols.push(col_name.to_string());
                }
            }
        }
        let cols_joined = if cols.is_empty() { "*".to_string() } else { cols.join("\n   ,") };
        return Ok(format!("SELECT TOP 1000\n    {}\nFROM\n    {}", cols_joined, clean_id));
    } else {
        let def_sql = format!(
            "SELECT m.definition FROM sys.objects AS o JOIN sys.sql_modules AS m ON o.object_id = m.object_id WHERE o.object_id = OBJECT_ID('{}')",
            clean_id
        );
        let stream = client.simple_query(&def_sql).await?;
        let result_sets = stream.into_results().await?;
        for rs in result_sets {
            for row in rs {
                if let Ok(Some(def)) = row.try_get::<&str, _>(0) {
                    return Ok(def.to_string());
                }
            }
        }
    }

    Err(anyhow!("Definition not found for object: {}", object_id))
}
