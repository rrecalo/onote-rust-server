use std::fmt;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use crate::model::folder::Folder;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Prefs{
        editorWidth: String,
        editorPosition: String
}


#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct User {
    
    pub _id: ObjectId,
    pub email: String,
    pub notes: Vec<String>,
    pub folders: Vec<Folder>,
    pub prefs: Prefs,
    pub lastNote: String,

}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{0} | {1}", self._id, self.email)
    }
}
