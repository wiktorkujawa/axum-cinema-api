use ::chrono::{DateTime as ChronoDateTime, Utc};
use mongodb::bson::serde_helpers::serialize_bson_datetime_as_rfc3339_string;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::utils::serialize_object_id;

use super::{hall_model::Hall, movie_model::Movie};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Session {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_object_id"
    )]
    pub id: Option<ObjectId>,
    pub title: Option<String>,
    #[serde(
        serialize_with = "serialize_object_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub movie_id: Option<ObjectId>,
    #[serde(
        serialize_with = "serialize_object_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub hall_id: Option<ObjectId>,
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionResponse {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_object_id"
    )]
    pub id: Option<ObjectId>,
    pub title: Option<String>,
    #[serde(
        serialize_with = "serialize_object_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub movie_id: Option<ObjectId>,
    #[serde(
        serialize_with = "serialize_object_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub hall_id: Option<ObjectId>,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub start: DateTime,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub end: DateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionDetail {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_object_id"
    )]
    pub id: Option<ObjectId>,
    pub title: Option<String>,
    #[serde(
        serialize_with = "serialize_object_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub movie_id: Option<ObjectId>,
    #[serde(
        serialize_with = "serialize_object_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub hall_id: Option<ObjectId>,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub start: DateTime,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub end: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movie: Option<Movie>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hall: Option<Hall>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionUpdate {
    pub movie_id: Option<ObjectId>,
    pub hall_id: Option<ObjectId>,
    pub title: Option<String>,
    pub start: Option<ChronoDateTime<Utc>>,
    pub end: Option<ChronoDateTime<Utc>>,
    pub poster: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionDeleteResponse {
    pub message: String,
    pub id: String,
}
