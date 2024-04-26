use std::sync::Arc;

use crate::{
    models::{AppState, VerifyQuery},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use serde_json::json;

#[route(
get,
"/quests/hashstack/verify_twitter_rt",
crate::endpoints::quests::hashstack::verify_twitter_rw
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 139;
    match state.upsert_completed_task(query.addr, task_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
        Err(e) => get_error(format!("{}", e)),
    }
}
