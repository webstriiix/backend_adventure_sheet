use axum::{
    Json,
    extract::State,
    response::IntoResponse,
    http::StatusCode,
};
use serde_json::Value; // Added missing import
use crate::{
    db::AppState,
    error::Result,
    importers::full_json_importer,
};

pub async fn trigger_import(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse> {
    let content = serde_json::to_string(&payload).unwrap();

    full_json_importer::import_everything(&state.db, &content).await
        .map_err(|e| crate::error::AppError::Internal(e))?;

    Ok(StatusCode::OK)
}

pub async fn trigger_import_spell_classes(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse> {
    let content = serde_json::to_string(&payload).unwrap();

    full_json_importer::import_spell_classes(&state.db, &content).await
        .map_err(|e| crate::error::AppError::Internal(e))?;

    Ok(StatusCode::OK)
}
