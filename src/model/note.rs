use std::fmt;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Note {
    _id: ObjectId,
    title: String,
    text: String,
    folder: String,
    index: i32,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{0} | {1}", self._id, self.title)
    }
}

