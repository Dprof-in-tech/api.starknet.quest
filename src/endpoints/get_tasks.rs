use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document, Document};
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::{fmt::format, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTask {
    id: u32,
    name: String,
    href: String,
    cta: String,
    verify_endpoint: String,
    verify_endpoint_type: String,
    verify_redirect: Option<String>,
    desc: String,
    completed: bool,
}

#[derive(Deserialize)]
pub struct GetTasksQuery {
    quest_id: u32,
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetTasksQuery>,
) -> impl IntoResponse {
    let quests_collection = state.db.collection::<Document>("quests");
    let quest_document = quests_collection
        .find_one(doc! { "id": query.quest_id }, None)
        .await;

    match quest_document {
        Ok(Some(quest)) => {
            if let Ok(task_ids) = quest.get_array("task_ids") {
                let pipeline = vec![
                    doc! { "$match": { "id": { "$in": task_ids } } },
                    doc! {
                        "$lookup": {
                            "from": "completed_tasks",
                            "let": { "task_id": "$id" },
                            "pipeline": [
                                {
                                    "$match": {
                                        "$expr": { "$eq": [ "$task_id", "$$task_id" ] },
                                        "address": query.addr.to_string(),
                                    },
                                },
                            ],
                            "as": "completed",
                        }
                    },
                    doc! {
                        "$project": {
                            "_id": 0,
                            "id":  1,
                            "name": 1,
                            "href": 1,
                            "cta": 1,
                            "verify_endpoint": 1,
                            "verify_redirect" : 1,
                            "verify_endpoint_type": 1,
                            "desc": 1,
                            "completed": { "$gt": [ { "$size": "$completed" }, 0 ] },
                        }
                    },
                ];
                let tasks_collection = state.db.collection::<Document>("tasks");
                match tasks_collection.aggregate(pipeline, None).await {
                    Ok(mut cursor) => {
                        let mut tasks: Vec<UserTask> = Vec::new();
                        while let Some(result) = cursor.next().await {
                            match result {
                                Ok(document) => {
                                    if let Ok(task) = from_document::<UserTask>(document) {
                                        tasks.push(task);
                                    }
                                }
                                _ => continue,
                            }
                        }
                        if tasks.is_empty() {
                            get_error("No tasks found for this quest_id".to_string())
                        } else {
                            tasks.sort_by(|a, b| a.id.cmp(&b.id));
                            (StatusCode::OK, Json(tasks)).into_response()
                        }
                    }
                    Err(_) => get_error("Error querying tasks".to_string()),
                }
            } else {
                get_error("Quest does not have a task_ids field".to_string())
            }
        }
        Ok(None) => get_error("No quest found with this quest_id".to_string()),
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
