use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Folder {
    pub _id: String,
    pub name: String,
    pub order: i32,
    pub opened: bool

}
