use std::fmt;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use crate::model::folder::Folder;

#[derive(Serialize, Deserialize)]
pub struct Prefs{
        #[allow(non_snake_case)]
        editorWidth: String,
        #[allow(non_snake_case)]
        editorPosition: String
}


#[derive(Serialize, Deserialize)]
pub struct User {

    

    pub _id: ObjectId,
    pub email: String,
    pub notes: Vec<String>,
    pub folders: Vec<Folder>,
    pub prefs: Prefs,
    #[allow(non_snake_case)]
    pub lastNote: String,

}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{0} | {1}", self._id, self.email)
    }
}
