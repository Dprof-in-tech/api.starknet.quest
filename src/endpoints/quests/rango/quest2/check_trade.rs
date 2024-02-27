use std::sync::Arc;

use crate::models::VerifyQuery;
use crate::utils::{to_hex, CompletedTasksTrait};
use crate::{models::AppState, utils::get_error};
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
    "/quests/rango/quest2/check_trade",
    crate::endpoints::quests::rango::quest2::check_trade
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 126;
    let mut address_hex = to_hex(query.addr);

    // remove "0x"
    address_hex.remove(0);
    address_hex.remove(0);

    // remove leading zeroes
    while address_hex.starts_with("0") {
        address_hex.remove(0);
    }

    // add "0x" back
    address_hex.insert(0, 'x');
    address_hex.insert(0, '0');

    let res = make_rango_request(
        &state.conf.rango.api_endpoint,
        &state.conf.rango.api_key,
        &address_hex,
    )
    .await;
    let response = match res {
        Ok(response) => response,
        Err(_) => return get_error(format!("Try again later")),
    };
    if let Some(_) = response.get("data") {
        let data = response.get("data").unwrap();
        if let Some(res) = data.get("result") {
            if res.as_bool().unwrap() {
                return match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                };
            }
        }
    }
    get_error("User has not completed the task".to_string())
}

async fn make_rango_request(
    endpoint: &str,
    api_key: &str,
    addr: &str,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    match client
        .post(endpoint)
        .json(&json!({
            "address": addr,
        }))
        .header("apiKey", api_key)
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => Ok(json),
            Err(_) => Err(format!("Funds not bridged")),
        },
        Err(_) => Err(format!("Funds not bridged")),
    }
}
