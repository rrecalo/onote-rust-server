use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Folder {
    _id: String,
    name: String,
    order: i32,
    opened: bool

}
