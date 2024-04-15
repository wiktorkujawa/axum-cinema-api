use serde::Serializer;
use mongodb::bson::oid::ObjectId;

pub fn serialize_object_id<S>(id: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match id {
        Some(id) => serializer.serialize_str(&id.to_hex()),
        None => serializer.serialize_none(),
    }
}