use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

use crate::utils::serialize_object_id;

use super::{movie_model::Movie, session_model::SessionResponse};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hall {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none", serialize_with = "serialize_object_id")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub capacity: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HallDetail {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none", serialize_with = "serialize_object_id")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub capacity: u32,
    pub description: String,
    pub movies: Vec<Movie>,
    pub sessions: Vec<SessionResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct HallUpdate {
    pub name: Option<String>,
    pub capacity: Option<u32>,
    pub description: Option<String>,
}