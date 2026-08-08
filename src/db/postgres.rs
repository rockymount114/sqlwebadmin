use anyhow::Result;
use serde_json::Value;
use sqlx::{Column, PgPool, Row, ValueRef};
use std::time::Instant;
use crate::models::{QueryResultTable, QueryExecuteResponse, SchemaNode};

pub async fn execute_query(connection_string: &str, query: &str) -> QueryExecuteResponse {
    let start = Instant::now();
    let pool = match PgPool::connect(connection_string).await {
        Ok(p) => p,
        Err(e) => {
            return QueryExecuteResponse {
                success: false,
                tables: vec![],
                total_affected_rows: 0,
                execution_time_ms: start.elapsed().as_millis(),
                error: Some(format!("PostgreSQL Connection Error: {}", e)),
                messages: vec![],
            };
        }
    };

    match sqlx::query(query).fetch_all(&pool).await {
        Ok(rows) => {
            let mut col_names = Vec::new();
            if !rows.is_empty() {
                for col in rows[0].columns() {
                    col_names.push(col.name().to_string());
                }
            }

            let mut table_rows = Vec::new();
            let row_count = rows.len() as u64;

            for row in rows {
                let mut row_values = Vec::new();
                for (i, _col) in row.columns().iter().enumerate() {
                    let val_ref = row.try_get_raw(i);
                    let val = match val_ref {
                        Ok(v) if v.is_null() => Value::Null,
                        Ok(_) => {
                            if let Ok(s) = row.try_get::<String, _>(i) {
                                Value::String(s)
                            } else if let Ok(i_val) = row.try_get::<i64, _>(i) {
                                Value::from(i_val)
                            } else if let Ok(f_val) = row.try_get::<f64, _>(i) {
                                Value::from(f_val)
                            } else if let Ok(b_val) = row.try_get::<bool, _>(i) {
                                Value::Bool(b_val)
                            } else if let Ok(j_val) = row.try_get::<serde_json::Value, _>(i) {
                                j_val
                            } else {
                                Value::String("<unsupported binary/custom type>".to_string())
                            }
                        }
                        Err(_) => Value::Null,
                    };
                    row_values.push(val);
                }
                table_rows.push(row_values);
            }

            QueryExecuteResponse {
                success: true,
                tables: vec![QueryResultTable {
                    columns: col_names,
                    rows: table_rows,
                    affected_rows: row_count,
                }],
                total_affected_rows: row_count,
                execution_time_ms: start.elapsed().as_millis(),
                error: None,
                messages: vec![format!("Query executed successfully in {} ms.", start.elapsed().as_millis())],
            }
        }
        Err(e) => QueryExecuteResponse {
            success: false,
            tables: vec![],
            total_affected_rows: 0,
            execution_time_ms: start.elapsed().as_millis(),
            error: Some(format!("PostgreSQL Execution Error: {}", e)),
            messages: vec![],
        },
    }
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
            text: "Functions / Procedures".to_string(),
            node_type: "SPROC".to_string(),
            value: "SPROC".to_string(),
            has_children: true,
        },
    ])
}

pub async fn get_children(connection_string: &str, node_type: &str, parent_id: &str) -> Result<Vec<SchemaNode>> {
    let pool = PgPool::connect(connection_string).await?;
    let mut nodes = Vec::new();

    match node_type {
        "TABLE" => {
            let rows = sqlx::query("SELECT table_schema || '.' || table_name AS full_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_schema NOT IN ('pg_catalog', 'information_schema') ORDER BY table_schema, table_name")
                .fetch_all(&pool)
                .await?;
            for r in rows {
                let name: String = r.get(0);
                nodes.push(SchemaNode {
                    id: format!("TABLE.{}", name),
                    text: name.clone(),
                    node_type: "TABLE_ITEM".to_string(),
                    value: format!("TABLE.{}", name),
                    has_children: true,
                });
            }
        }
        "VIEW" => {
            let rows = sqlx::query("SELECT table_schema || '.' || table_name AS full_name FROM information_schema.views WHERE table_schema NOT IN ('pg_catalog', 'information_schema') ORDER BY table_schema, table_name")
                .fetch_all(&pool)
                .await?;
            for r in rows {
                let name: String = r.get(0);
                nodes.push(SchemaNode {
                    id: format!("VIEW.{}", name),
                    text: name.clone(),
                    node_type: "VIEW_ITEM".to_string(),
                    value: format!("VIEW.{}", name),
                    has_children: true,
                });
            }
        }
        "SPROC" => {
            let rows = sqlx::query("SELECT routine_schema || '.' || routine_name AS full_name FROM information_schema.routines WHERE routine_schema NOT IN ('pg_catalog', 'information_schema') ORDER BY routine_schema, routine_name")
                .fetch_all(&pool)
                .await?;
            for r in rows {
                let name: String = r.get(0);
                nodes.push(SchemaNode {
                    id: format!("SPROC.{}", name),
                    text: name.clone(),
                    node_type: "SPROC_ITEM".to_string(),
                    value: format!("SPROC.{}", name),
                    has_children: false,
                });
            }
        }
        "TABLE_ITEM" | "VIEW_ITEM" => {
            let clean_id = parent_id.trim_start_matches("TABLE.").trim_start_matches("VIEW.");
            let parts: Vec<&str> = clean_id.split('.').collect();
            let (schema, table) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                ("public", clean_id)
            };
            let rows = sqlx::query("SELECT column_name, data_type, character_maximum_length FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position")
                .bind(schema)
                .bind(table)
                .fetch_all(&pool)
                .await?;
            for r in rows {
                let col: String = r.get(0);
                let dtype: String = r.get(1);
                let len: Option<i32> = r.get(2);
                let display_len = len.map_or("".to_string(), |l| format!("({})", l));
                nodes.push(SchemaNode {
                    id: format!("COLUMN.{}.{}", clean_id, col),
                    text: format!("{} ({}{})", col, dtype, display_len),
                    node_type: "COLUMN".to_string(),
                    value: format!("COLUMN.{}.{}", clean_id, col),
                    has_children: false,
                });
            }
        }
        _ => {}
    }

    Ok(nodes)
}

pub async fn get_definition(connection_string: &str, node_type: &str, object_id: &str) -> Result<String> {
    let pool = PgPool::connect(connection_string).await?;
    let clean_id = object_id.trim_start_matches("TABLE.").trim_start_matches("VIEW.").trim_start_matches("SPROC.");

    if node_type == "TABLE_ITEM" || node_type == "TABLE" {
        return Ok(format!("SELECT * FROM {} LIMIT 1000;", clean_id));
    } else if node_type == "VIEW_ITEM" {
        let sql = format!("SELECT pg_get_viewdef('{}', true);", clean_id);
        let row = sqlx::query(&sql).fetch_one(&pool).await?;
        let def: String = row.get(0);
        return Ok(def);
    } else if node_type == "SPROC_ITEM" {
        let sql = format!("SELECT pg_get_functiondef(oid) FROM pg_proc WHERE proname = '{}';", clean_id);
        let row = sqlx::query(&sql).fetch_one(&pool).await?;
        let def: String = row.get(0);
        return Ok(def);
    }

    Err(anyhow::anyhow!("Definition not found for object: {}", object_id))
}
