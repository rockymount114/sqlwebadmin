use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use csv::Writer;
use serde_json::json;
use std::sync::Arc;

use crate::{
    config::AppConfig,
    db,
    models::{
        ConnectTestRequest, DbDriver, GetChildrenQuery, GetDatabasesQuery, GetDefinitionQuery,
        GetTreeQuery, QueryRequest,
    },
};

pub struct AppState {
    pub config: AppConfig,
}

pub async fn get_config_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!({
        "default_connection_string": state.config.default_connection_string,
        "default_driver": state.config.default_driver,
        "port": state.config.port,
    }))
}

pub async fn test_connection_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ConnectTestRequest>,
) -> impl IntoResponse {
    let driver = DbDriver::from_str(&payload.driver);
    let test_query = match driver {
        DbDriver::Mssql | DbDriver::Postgres | DbDriver::Sqlite => "SELECT 1",
        DbDriver::Mysql => "SELECT 1",
    };

    let result = db::execute_query(&driver, &payload.connection_string, None, test_query).await;
    if result.success {
        Json(json!({
            "success": true,
            "message": "Successfully connected to database!"
        }))
    } else {
        Json(json!({
            "success": false,
            "message": result.error.unwrap_or_else(|| "Failed to connect to database".to_string())
        }))
    }
}

pub async fn get_databases_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetDatabasesQuery>,
) -> impl IntoResponse {
    let driver_str = params.driver.as_deref().unwrap_or(&state.config.default_driver);
    let conn_str = params.connection_string.as_deref().unwrap_or(&state.config.default_connection_string);
    let driver = DbDriver::from_str(driver_str);

    match db::get_databases(&driver, conn_str).await {
        Ok(dbs) => Json(json!({ "success": true, "databases": dbs })).into_response(),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

pub async fn schema_tree_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetTreeQuery>,
) -> impl IntoResponse {
    let driver_str = params.driver.as_deref().unwrap_or(&state.config.default_driver);
    let conn_str = params.connection_string.as_deref().unwrap_or(&state.config.default_connection_string);
    let driver = DbDriver::from_str(driver_str);
    let db_opt = params.database.as_deref();
    let all_dbs = params.all_databases.unwrap_or(false);

    match db::get_tree_root(&driver, conn_str, db_opt, all_dbs).await {
        Ok(nodes) => Json(json!({ "success": true, "nodes": nodes })).into_response(),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

pub async fn schema_children_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetChildrenQuery>,
) -> impl IntoResponse {
    let driver_str = params.driver.as_deref().unwrap_or(&state.config.default_driver);
    let conn_str = params.connection_string.as_deref().unwrap_or(&state.config.default_connection_string);
    let driver = DbDriver::from_str(driver_str);

    match db::get_children(&driver, conn_str, &params.node_type, &params.parent_id).await {
        Ok(nodes) => Json(json!({ "success": true, "nodes": nodes })).into_response(),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

pub async fn schema_definition_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetDefinitionQuery>,
) -> impl IntoResponse {
    let driver_str = params.driver.as_deref().unwrap_or(&state.config.default_driver);
    let conn_str = params.connection_string.as_deref().unwrap_or(&state.config.default_connection_string);
    let driver = DbDriver::from_str(driver_str);

    match db::get_definition(&driver, conn_str, &params.node_type, &params.object_id).await {
        Ok(def) => Json(json!({ "success": true, "definition": def })).into_response(),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

pub async fn execute_query_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    let driver_str = payload.driver.as_deref().unwrap_or(&state.config.default_driver);
    let conn_str = payload.connection_string.as_deref().unwrap_or(&state.config.default_connection_string);
    let driver = DbDriver::from_str(driver_str);
    let db_opt = payload.database.as_deref();

    if payload.query.trim().is_empty() {
        return Json(json!({
            "success": false,
            "tables": [],
            "total_affected_rows": 0,
            "execution_time_ms": 0,
            "error": "Query cannot be blank",
            "messages": []
        }));
    }

    let response = db::execute_query(&driver, &conn_str, db_opt, &payload.query).await;
    Json(json!(response))
}

pub async fn export_query_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> Response {
    let driver_str = payload.driver.as_deref().unwrap_or(&state.config.default_driver);
    let conn_str = payload.connection_string.as_deref().unwrap_or(&state.config.default_connection_string);
    let driver = DbDriver::from_str(driver_str);
    let db_opt = payload.database.as_deref();

    if payload.query.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Query cannot be blank").into_response();
    }

    let res = db::execute_query(&driver, &conn_str, db_opt, &payload.query).await;
    if !res.success || res.tables.is_empty() {
        let err_msg = res.error.unwrap_or_else(|| "Query returned no result to export".to_string());
        return (StatusCode::BAD_REQUEST, err_msg).into_response();
    }

    let table = &res.tables[0];
    let mut wtr = Writer::from_writer(Vec::new());

    if let Err(_) = wtr.write_record(&table.columns) {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write CSV header").into_response();
    }

    for row in &table.rows {
        let string_row: Vec<String> = row
            .iter()
            .map(|val| match val {
                serde_json::Value::Null => "".to_string(),
                serde_json::Value::String(s) => s.clone(),
                v => v.to_string(),
            })
            .collect();
        if let Err(_) = wtr.write_record(&string_row) {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write CSV row").into_response();
        }
    }

    let csv_bytes = match wtr.into_inner() {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode CSV").into_response(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/csv; charset=utf-8".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"query_export.csv\"".parse().unwrap(),
    );

    (headers, csv_bytes).into_response()
}
