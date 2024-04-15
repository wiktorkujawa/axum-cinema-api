use crate::models::hall_model::{Hall, HallDetail, HallUpdate};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, Bson, Document},
    Client,
};
use serde_json::Value;
use std::sync::Arc;

pub async fn load_halls_with_details(
    Extension(client): Extension<Arc<Client>>,
) -> Result<Json<Vec<Hall>>, StatusCode> {
    let db = client.database("cinema-axum");
    let halls_collection = db.collection::<Hall>("halls");

    let pipeline = vec![];

    let mut cursor = match halls_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut result: Vec<Hall> = Vec::new();
    while let Some(doc) = cursor.try_next().await.expect("Failed to iterate") {
        let hall_detail: Hall = match from_document(doc) {
            Ok(hall) => hall,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        result.push(hall_detail);
    }

    Ok(Json(result))
}

pub async fn load_hall_with_details(
    Path(id_str): Path<String>,
    Extension(client): Extension<Arc<Client>>,
) -> Result<Json<HallDetail>, StatusCode> {
    let db = client.database("cinema-axum");
    let halls_collection = db.collection::<Hall>("halls");

    let hall_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let pipeline = vec![
        doc! {
            "$match": { "_id": hall_id }
        },
        doc! {
            "$lookup": {
                "from": "sessions",
                "localField": "_id",
                "foreignField": "hall_id",
                "as": "sessions"
            }
        },
        doc! {
            "$lookup": {
                "from": "movies",
                "let": { "movie_id": "$sessions.movie_id" },
                "pipeline": [
                    { "$match": { "$expr": { "$in": [ "$_id", "$$movie_id" ] } } }
                ],
                "as": "movies"
            }
        },
    ];

    let mut cursor = match halls_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let Some(doc) = cursor.try_next().await.expect("Failed to iterate") {
        let hall_detail: HallDetail = match from_document(doc) {
            Ok(detail) => detail,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(Json(hall_detail))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn add_hall(
    Extension(client): Extension<Arc<Client>>,
    AxumJson(hall): AxumJson<Hall>,
) -> Result<Json<Hall>, StatusCode> {
    let db = client.database("cinema-axum");
    let halls_collection = db.collection::<Hall>("halls");

    match halls_collection.insert_one(hall.clone(), None).await {
        Ok(_) => Ok(Json(hall)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_hall(
    Extension(client): Extension<Arc<Client>>,
    axum::extract::Path(id_str): axum::extract::Path<String>,
    Json(update_data): Json<HallUpdate>,
) -> Result<Json<HallUpdate>, StatusCode> {
    let db = client.database("cinema-axum");
    let halls_collection = db.collection::<Document>("halls");

    let hall_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let json =
        serde_json::to_value(&update_data).unwrap_or_else(|_| Value::Object(Default::default()));

    let mut update_doc = Document::new();
    if let Value::Object(obj) = json {
        for (key, value) in obj {
            if !value.is_null() {
                let bson_value = match Bson::try_from(value) {
                    Ok(bv) => bv,
                    Err(_) => continue,
                };
                update_doc.insert(key, bson_value);
            }
        }
    }

    let update = doc! {
        "$set": update_doc,
    };

    match halls_collection
        .update_one(doc! {"_id": hall_id}, update, None)
        .await
    {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                Ok(Json(update_data))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_hall(
    Path(id_str): Path<String>,
    Extension(client): Extension<Arc<Client>>,
) -> Result<Json<String>, StatusCode> {
    let db = client.database("cinema-axum");
    let halls_collection = db.collection::<Hall>("halls");
    let sessions_collection = db.collection::<Document>("sessions");

    let hall_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match halls_collection
        .delete_one(doc! {"_id": hall_id}, None)
        .await
    {
        Ok(delete_result) => {
            if delete_result.deleted_count == 1 {
                let update_result = sessions_collection.update_many(
                    doc! {"hall_id": hall_id.to_string()},
                    doc! {"$set": {"hall_id": null}},
                    None
                ).await;
                
                match update_result {
                    Ok(update_result) => {
                        if update_result.modified_count > 0 {
                            println!("Successfully updated {} session(s).", update_result.modified_count);
                            Ok(Json("Hall ID set to null in associated sessions successfully".to_string()))
                        } else {
                            println!("No sessions found for hall_id: {}", hall_id.to_string());
                            Err(StatusCode::NOT_FOUND)
                        }
                    },
                    Err(e) => {
                        eprintln!("Error updating sessions: {:?}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
