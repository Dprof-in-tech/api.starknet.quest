use crate::models::QuizDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuizQuery {
    id: String,
    // addr could be used as entropy for sending a server side randomized order
    // let's keep on client side for now
    #[allow(dead_code)]
    addr: FieldElement,
}

pub_struct!(Clone, Serialize; QuizQuestionResp {
    kind: String,
    layout: String,
    question: String,
    options: Vec<String>,
    image_for_layout: Option<String>
});

#[derive(Clone, Serialize)]
pub struct QuizResponse {
    name: String,
    desc: String,
    questions: Vec<QuizQuestionResp>,
}

#[route(get, "/get_quiz", crate::endpoints::get_quiz)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuizQuery>,
) -> impl IntoResponse {
    let id = query.id.to_string();
    let collection = state.db.collection::<QuizDocument>("quizzes");
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "id": &id
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "quiz_questions",
                "let": doc! {
                    "id": "$id"
                },
                "pipeline": [
                    doc! {
                        "$match": doc! {
                            "quiz_id": &id
                        }
                    },
                    doc! {
                        "$project": doc! {
                            "correct_answers": 0,
                            "quiz_id": 0,
                            "_id": 0
                        }
                    }
                ],
                "as": "questions"
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "id": 0
            }
        },
    ];

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        return (StatusCode::OK, Json(document)).into_response();
                    }
                    Err(e) => {
                        return get_error(e.to_string());
                    }
                }
            }
            get_error("Quiz not found".to_string())
        }
        Err(e) => {
            return get_error(e.to_string());
        }
    }
}
