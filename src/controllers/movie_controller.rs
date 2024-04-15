use axum::{
    extract::{Extension, Path}, http::StatusCode, response::Json, Json as AxumJson
};
use mongodb::{bson::{doc, from_document, oid::ObjectId, Bson, Document}, Client};
use std::sync::Arc;
use crate::models::movie_model::{Movie, MovieDetail, MovieUpdate};
use futures::TryStreamExt;
use serde_json::Value;

pub async fn load_movies_with_details(Extension(client): Extension<Arc<Client>>) -> Result<Json<Vec<Movie>>, StatusCode> {
    let db = client.database("cinema-axum");
    let movies_collection = db.collection::<Movie>("movies");

    let pipeline = vec![];

    let mut cursor = match movies_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut result: Vec<Movie> = Vec::new();
    while let Some(doc) = cursor.try_next().await.expect("Failed to iterate") {
        let movie_detail: Movie = match from_document(doc) {
            Ok(movie) => movie,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        result.push(movie_detail);
    }

    Ok(Json(result))
}

pub async fn load_movie_with_details(
    Path(id_str): Path<String>,
    Extension(client): Extension<Arc<Client>>,
) -> Result<Json<MovieDetail>, StatusCode> {
    let db = client.database("cinema-axum");
    let movies_collection = db.collection::<Movie>("movies");

    let movie_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let pipeline = vec![
        doc! {
            "$match": { "_id": movie_id }
        },
        doc! {
            "$lookup": {
                "from": "sessions",
                "localField": "_id",
                "foreignField": "movie_id",
                "as": "sessions"
            }
        },
        doc! {
            "$lookup": {
                "from": "halls",
                "let": { "hall_id": "$sessions.hall_id" },
                "pipeline": [
                    { "$match": { "$expr": { "$in": [ "$_id", "$$hall_id" ] } } }
                ],
                "as": "halls"
            }
        },
    ];

    let mut cursor = match movies_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let Some(doc) = cursor.try_next().await.expect("Failed to iterate") {
        let movie_detail: MovieDetail = match from_document(doc) {
            Ok(detail) => detail,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(Json(movie_detail))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn add_movie(
    Extension(client): Extension<Arc<Client>>,
    AxumJson(movie): AxumJson<Movie>,
) -> Result<Json<Movie>, StatusCode> {
    let db = client.database("cinema-axum");
    let movies_collection = db.collection::<Movie>("movies");

    match movies_collection.insert_one(movie.clone(), None).await {
        Ok(_) => Ok(Json(movie)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_movie(
    Path(id_str): Path<String>,
    Extension(client): Extension<Arc<Client>>,
) -> Result<Json<String>, StatusCode> {
    let db = client.database("cinema-axum");
    let movies_collection = db.collection::<Movie>("movies");

    let movie_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match movies_collection.delete_one(doc! {"_id": movie_id}, None).await {
        Ok(delete_result) => {
            if delete_result.deleted_count == 1 {
                Ok(Json("Movie deleted successfully".to_string()))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_movie(
    Extension(client): Extension<Arc<Client>>,
    axum::extract::Path(id_str): axum::extract::Path<String>,
    Json(update_data): Json<MovieUpdate>,
) -> Result<Json<MovieUpdate>, StatusCode> {
    let db = client.database("cinema-axum");
    let movies_collection = db.collection::<Document>("movies");

    let movie_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let json = serde_json::to_value(&update_data).unwrap_or_else(|_| Value::Object(Default::default()));

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

    match movies_collection.update_one(doc! {"_id": movie_id}, update, None).await {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                Ok(Json(update_data))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}