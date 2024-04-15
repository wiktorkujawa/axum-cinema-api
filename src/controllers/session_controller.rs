use axum::{
    extract::{Extension, Path}, http::StatusCode, response::Json
};
use mongodb::{bson::{self, doc, from_document, oid::ObjectId, DateTime}, Client};
use chrono::{ DateTime as ChronoDateTime, Utc };
use std::sync::Arc;
use crate::models::session_model::{Session, SessionDetail, SessionResponse, SessionUpdate};
use futures::TryStreamExt;

async fn is_hall_available(
    client: &Arc<Client>,
    hall_id: &ObjectId,
    start: &Option<ChronoDateTime<Utc>>,
    end: &Option<ChronoDateTime<Utc>>,
    exclude_session_id: Option<ObjectId>,
) -> bool {
    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<SessionUpdate>("sessions");

    let start_bson = start.map(|dt| bson::DateTime::from_chrono(dt)).unwrap();
    let end_bson = end.map(|dt| bson::DateTime::from_chrono(dt)).unwrap();


    let mut query = doc! {
        "hall_id": hall_id,
        "$and": [
            { "start": { "$lt": end_bson } },
            { "end": { "$gt": start_bson } },
        ],
    };

    if let Some(exclude_id) = exclude_session_id {
        query.insert("_id", doc! { "$ne": exclude_id });
    }

    let count = sessions_collection.count_documents(query, None).await.unwrap_or(0);

    count == 0
}


pub async fn load_sessions_with_details(Extension(client): Extension<Arc<Client>>) -> Result<Json<Vec<SessionDetail>>, StatusCode> {
    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<Session>("sessions");

    let pipeline = vec![
        doc! {
            "$lookup": {
                "from": "movies",
                "localField": "movie_id",
                "foreignField": "_id",
                "as": "movie",
            },
        },
        doc! {
            "$lookup": {
                "from": "halls",
                "localField": "hall_id",
                "foreignField": "_id",
                "as": "hall"
            }
        },
        doc! {
            "$unwind": {
                "path": "$movie",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$unwind": {
                "path": "$hall",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$project": {
                "movie.description": 0,
                "movie.poster": 0,
            }
        },
    ];

    let mut cursor = match sessions_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    

    let mut result: Vec<SessionDetail> = Vec::new();
    while let Some(doc) = cursor.try_next().await.expect("Failed to iterate") {
        let session_detail: SessionDetail = match from_document(doc) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("Aggregation error: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            },
        };
        result.push(session_detail);
    }

    Ok(Json(result))
}

pub async fn fetch_session_by_id(
    Path(id_str): Path<String>,
    Extension(client): Extension<Arc<Client>>,
) -> Result<Json<Option<SessionDetail>>, StatusCode> {
    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<Session>("sessions");

    let id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let pipeline = vec![
        doc! {
            "$match": {
                "_id": id,
            }
        },
        doc! {
            "$lookup": {
                "from": "movies",
                "localField": "movie_id",
                "foreignField": "_id",
                "as": "movie",
            },
        },
        doc! {
            "$lookup": {
                "from": "halls",
                "localField": "hall_id",
                "foreignField": "_id",
                "as": "hall"
            }
        },
        doc! {
            "$unwind": {
                "path": "$movie",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$unwind": {
                "path": "$hall",
                "preserveNullAndEmptyArrays": true
            }
        },
    ];

    let mut cursor = match sessions_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match cursor.try_next().await {
        Ok(Some(doc)) => {
            let session: Result<SessionDetail, _> = from_document(doc);
            match session {
                Ok(session) => Ok(Json(Some(session))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        Ok(None) => Ok(Json(None)), 
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_sessions(Extension(client): Extension<Arc<Client>>) -> Result<Vec<SessionDetail>, StatusCode> {
    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<Session>("sessions");

    let pipeline = vec![
        doc! {
            "$lookup": {
                "from": "movies",
                "localField": "movie_id",
                "foreignField": "_id",
                "as": "movie",
            },
        },
        doc! {
            "$lookup": {
                "from": "halls",
                "localField": "hall_id",
                "foreignField": "_id",
                "as": "hall"
            }
        },
        doc! {
            "$unwind": {
                "path": "$movie",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$unwind": {
                "path": "$hall",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$project": {
                "movie.description": 0,
                "movie.poster": 0,
            }
        },
    ];

    let mut cursor = match sessions_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    

    let mut result: Vec<SessionDetail> = Vec::new();
    while let Some(doc) = cursor.try_next().await.expect("Failed to iterate") {
        let session_detail: SessionDetail = match from_document(doc) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("Aggregation error: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            },
        };
        result.push(session_detail);
    }

    Ok(result)
}



pub async fn add_ws_session(
    Extension(client): Extension<Arc<Client>>,
    Json(session_data): Json<SessionUpdate>,
) -> Result<SessionResponse, StatusCode> {
    if session_data.end <= session_data.start {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(hall_id) = session_data.hall_id {
        if !is_hall_available(&client, &hall_id, &session_data.start, &session_data.end, None).await {
            return Err(StatusCode::CONFLICT);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<Session>("sessions");

    let session_to_insert = Session {
        id: None,
        title: session_data.title,
        movie_id: session_data.movie_id,
        hall_id: session_data.hall_id,
        start: DateTime::from_millis(session_data.start.unwrap().timestamp_millis()),
        end: DateTime::from_millis(session_data.end.unwrap().timestamp_millis()),
    };

    match sessions_collection.insert_one(&session_to_insert, None).await {
        Ok(insert_result) => {
            let created_session = SessionResponse {
                id: Some(insert_result.inserted_id.as_object_id().unwrap()),
                title: session_to_insert.title,
                movie_id: session_to_insert.movie_id,
                hall_id: session_to_insert.hall_id,
                start: session_to_insert.start,
                end: session_to_insert.end              
            };
            Ok(created_session)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_ws_session(
    Extension(client): Extension<Arc<Client>>,
    axum::extract::Path(id_str): axum::extract::Path<String>,
    Json(session_data): Json<SessionUpdate>,
) -> Result<SessionResponse, StatusCode> {
    let session_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<Session>("sessions");

    let current_session = sessions_collection.find_one(doc! {"_id": session_id}, None).await.unwrap();
    let hall_id = current_session.as_ref().and_then(|doc| doc.hall_id.clone()).or(session_data.hall_id.clone()).ok_or(StatusCode::BAD_REQUEST)?;

    if let (Some(start), Some(end)) = (session_data.start, session_data.end) {
        if end <= start {
            return Err(StatusCode::BAD_REQUEST);
        }

        if !is_hall_available(&client, &hall_id, &Some(start), &Some(end), Some(session_id)).await {
            return Err(StatusCode::CONFLICT);
        }
    }

    let update_doc = doc! {
        "$set": {
            "title": session_data.title,
            "movie_id": session_data.movie_id,
            "hall_id": hall_id,
            "start": session_data.start,
            "end": session_data.end,
        }
    };

    sessions_collection.update_one(doc! {"_id": session_id}, update_doc, None).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated_session = sessions_collection.find_one(doc! {"_id": session_id}, None).await.unwrap().expect("Session not found after update");

    let response = SessionResponse {
        id: Some(updated_session.id.unwrap()),
        title: updated_session.title,
        movie_id: updated_session.movie_id,
        hall_id: updated_session.hall_id,
        start: updated_session.start,
        end: updated_session.end
    };

    Ok(response)
}

pub async fn delete_ws_session(
    Extension(client): Extension<Arc<Client>>,
    axum::extract::Path(id_str): axum::extract::Path<String>,
) -> Result<StatusCode, StatusCode> {
    let session_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let db = client.database("cinema-axum");
    let sessions_collection = db.collection::<Session>("sessions");

    match sessions_collection.delete_one(doc! {"_id": session_id}, None).await {
        Ok(delete_result) => {
            if delete_result.deleted_count == 1 {
                Ok(StatusCode::OK)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
