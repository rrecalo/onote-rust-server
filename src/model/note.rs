use std::fmt;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub _id: ObjectId,
    pub title: String,
    pub text: String,
    pub folder: String,
    pub index: i32,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{0} | {1}", self._id, self.title)
    }
}

