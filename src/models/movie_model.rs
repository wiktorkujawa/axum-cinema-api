use serde::{Deserialize, Serialize };
use mongodb::bson::oid::ObjectId;

use crate::utils::serialize_object_id;

use super::{hall_model::Hall, session_model::SessionResponse};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Movie {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none", serialize_with = "serialize_object_id")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieDetail {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none", serialize_with = "serialize_object_id")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
    pub halls: Vec<Hall>,
    pub sessions: Vec<SessionResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct MovieUpdate {
    pub title: Option<String>,
    pub duration: Option<i32>,
    pub description: Option<String>,
    pub poster: Option<String>
}

