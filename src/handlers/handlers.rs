use std::str::FromStr;
use std::fmt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use axum::{
    response::IntoResponse,
    Extension,
    extract::{Path, Query},
    Json
};
use futures::stream::StreamExt;

use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    Client,
};

use crate::model::{note::Note, user::User, folder::Folder};


pub async fn hello_world() -> String {
    
    String::from("Hello World!")
}

pub async fn get_all_notes(
    client: Extension<Client>
    ) -> impl IntoResponse {

    let collection = client.clone().database("notes-app").collection::<Note>("notes");
    let cursor = collection.find(doc! {}, None).await;
    let v: Vec<Note> = cursor.expect("REASON")
        .filter_map(|doc| async move { doc.ok() })
        .collect()
        .await;
    /*
     * Debug Purposes
    for item in v.iter() {
        println!("{}", item);
    }
    */
    Json(v)

    }

pub async fn get_user(
    client: Extension<Client>,
    Path(user_email): Path<String>
    ) -> Option<User> {

    let collection = client.clone().database("notes-app").collection::<User>("users");
    //match out break down the Result<User, Err> object into an Option<User>
    match collection.find_one(doc! {"email": user_email}, None).await{
        Ok(document) => document,
        /*
        Ok(None) => {
            println!("nothing found");
            return None;
        }
        */
        Err(e)=>{
            dbg!(&e);
            return None;

        }
    }
}

pub async fn get_user_by_email(
    client: Extension<Client>,
    Path(user_email): Path<String>
    ) -> impl IntoResponse {

    let collection = client.clone().database("notes-app").collection::<User>("users");
    //match out break down the Result<User, Err> object into an Option<User>
    let user = match collection.find_one(doc! {"email": user_email}, None).await{
        Ok(document) => Ok(Json(document)),
        /*
        Ok(None) => {
            println!("nothing found");
            return None;
        }
        */
        Err(e)=>{
            dbg!(&e);
            return Err(())

        }
    };
    user

}

#[derive(Serialize, Deserialize)]
pub struct NoteInterface {
    pub _id: String,
    pub title: String,
    pub text: String,
    pub folder: String,
    pub index: i32,
}
pub async fn get_notes_by_user(
    client: Extension<Client>,
    Path(user_email): Path<String>
    ) -> impl IntoResponse {

    //let start = Instant::now();
    //get the user object to receive its 'notes' array
    let user = get_user(client.clone(), axum::extract::Path(user_email)).await.expect("No User found by email");
    
    //create an array of ObjectIds from the _id Strings found in the user.notes attribute
    let mut note_ids: Vec<ObjectId> = Vec::new();
    for item in user.notes.iter() {
        note_ids.push(ObjectId::from_str(item).unwrap());
    }

    //query to get all the notes that are within the array of ObjectIds we just made
    let collection = client.clone().database("notes-app").collection::<Note>("notes");
    let cursor = collection.find(doc! {"_id": { "$in" : note_ids} }, None).await;
    let v: Vec<Note> = cursor.expect("reason")
        .filter_map(|doc| async move { doc.ok() })
        .collect()
        .await;
  
    let mut notes_vec: Vec<NoteInterface> = Vec::new();

    for note in v.iter(){
        notes_vec.push(
            NoteInterface{
                _id: note._id.to_hex(),
                title: note.title.clone(),
                text: note.text.clone(),
                folder: note.folder.clone(),
                index: note.index.clone()
            }
            );
    }


    /*
     * Debug
    for item in v.iter() {
        println!("{}", item);
    }
    */

    //let end = Instant::now();
    //println!("{}", (end-start).as_millis());

    Json(notes_vec)

    }

#[derive(Debug, serde::Serialize, Deserialize)]
pub struct CreateNote{
    email: String,
    note_title: String,
}

impl std::fmt::Display for CreateNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "{0} | {1}", self.email, self.note_title)
    }
}

pub fn random_note_name() -> String {
        let mut rng = rand::thread_rng();
        let nums: Vec<u8> = (0..4).map(|_| rng.gen_range(0..=9)).collect();
        let result = format!("NewNote{0}{1}{2}{3}", nums[0], nums[1], nums[2], nums[3]);

        result
    }

pub async fn create_user_note(
    client: Extension<Client>,
    Json(payload): Json<CreateNote>
    ) -> impl IntoResponse {
   
    let new_note_title = match &payload.note_title.is_empty() {
        true => random_note_name(), 
        false => String::from(payload.note_title)
    };

    let user_email = payload.email;

    //println!("{}", new_note_title);
    
    let new_note = Note {
        _id: ObjectId::new(),
        title: new_note_title.clone(),
        text: String::from(""),
        folder: String::from(""),
        index: -1, 
    };

    
    let inserted = match client.clone().database("notes-app").collection("notes")
        .insert_one(new_note, None).await {
            Ok(document) => Ok(Json(document)),
            Err(e) => {dbg!(e);Err(())}
        };
    match &inserted {
        Ok(json) => {
            
            let _update = match client.clone().database("notes-app").collection::<User>("users")
                .update_one(doc! {"email": user_email}, 
                            doc! { "$push": {"notes" : &json.0.inserted_id.as_object_id().unwrap().to_hex() } }, None).await {
                    Ok(document) => {
                        //println!("{}", document.modified_count);
                        Ok(Json(document))
                    }
                    Err(e) => {dbg!(e); Err(())}
                };
            
            //println!("{}",json.0.inserted_id);
        }
        
        Err(_) => println!("err!"),
    };
    Json(NoteInterface{
        _id: inserted.unwrap().inserted_id.as_object_id().unwrap().to_hex(),
        title: new_note_title.clone(),
        text: String::from(""),
        folder: String::from(""),
        index: -1
    })
    //inserted
}

#[derive(Deserialize)]
pub struct DeleteNote{
    email: String,
    _id: ObjectId
}

pub async fn delete_user_note(
    client: Extension<Client>,
    Query(payload): Query<DeleteNote>
    ) -> impl IntoResponse{
    
    let notes = client.clone().database("notes-app").collection::<Note>("notes");
    //let id = Bson::ObjectId(ObjectId::from_str(&payload._id).unwrap());
    let id = Bson::ObjectId(payload._id.clone());
    let delete_result = notes.delete_one(doc! {"_id": id}, None).await;
   
    let users = client.clone().database("notes-app").collection::<User>("users");
    //println!("{}", &payload._id);
    let _delete_from_user = users.find_one_and_update(doc! {"email": payload.email},
                                doc! {"$pull" : {"notes": payload._id.to_string()}}, None).await;
    //println!("{}", delete_from_user.unwrap().unwrap());
    Json(delete_result.unwrap())
}

#[derive(Deserialize)]
pub struct CreateFolder{
    email: String,
    folder_index: i32,
    folder_name: String,
}

pub async fn create_user_folder(
    client: Extension<Client>,
    Json(payload): Json<CreateFolder>
    ) -> impl IntoResponse{

    let users = client.clone().database("notes-app").collection::<User>("users");

    let new_folder = Bson::Document(doc!{
                                "_id": ObjectId::new().to_string(),
                                "name": payload.folder_name,
                                "order": payload.folder_index,
                                "opened": true,
                            });


    let _folder_update = users.update_one(doc! {"email": &payload.email},
                            doc! {"$push": {"folders": &new_folder
                                                    }}, None).await.unwrap();
    Json(new_folder)
}


#[derive(Deserialize)]
pub struct DeleteFolder{
    email: String,
    _id: ObjectId
}

pub async fn delete_user_folder(
    client: Extension<Client>,
    Json(payload): Json<DeleteFolder>
    ) -> impl IntoResponse{

    let notes = client.clone().database("notes-app").collection::<Note>("notes");
    let cursor = notes.find(doc! {"folder": payload._id.to_string() }, None).await;
    //store all the notes with the match folder id in a vec
    let v: Vec<Note> = cursor.expect("reason")
        .filter_map(|doc| async move { doc.ok() })
        .collect()
        .await;

    //make a new vec and collect JUST the _id from each note as a String
    // -- Bson won't be able to serialize our Vec<Note> and thus we wouldn't be able to use the
    // powerful $in operator.
    let mut id_vec: Vec<String> = Vec::new();
    for item in v.iter(){
        id_vec.push(item._id.to_string());
    }

    //delete the notes from the 'notes' collection
    let _deleted_note_count = notes.delete_many(doc! {"folder" : payload._id.to_string()}, None).await.unwrap();
    //println!("deleted {} notes", deleted_note_count.deleted_count);

    let users = client.clone().database("notes-app").collection::<User>("users");
    //delete the notes from the users' 'notes' attribute
    let _deleted_notes_from_user = users.update_one(doc! {"email":&payload.email},
    doc! {"$pull":{"notes": {"$in":id_vec}}}, None).await.unwrap();
    //println!("{}", deleted_notes_from_user.modified_count);

    //remove the folder from the users 'folders' attribute
    let deleted_folder_from_user = users.update_one(doc! {"email":&payload.email},
                                doc!{"$pull":{"folders": {"_id":payload._id.to_string()}}
                                                            }, None).await.unwrap();
    //println!("Deleted {0} folder from user : {1}", deleted_folder_from_user.modified_count, payload.email);    
    Json(deleted_folder_from_user)
}

#[derive(Deserialize)]
pub struct UpdateUserLastNote {
    email: String,
    last_note: String,
}

pub async fn update_user_last_note(
    client: Extension<Client>,
    Json(payload): Json<UpdateUserLastNote>
    ) -> impl IntoResponse {
    
    let users = client.clone().database("notes-app").collection::<User>("users");
    let update = users.update_one(doc! {"email": &payload.email}, doc! {"$set":{"lastNote": &payload.last_note}}, None).await.unwrap();

    //println!("Update {0} {1}'s lastNote to {2}", update.modified_count, payload.email, payload.lastNote);

    Json(update)
}

#[derive(Deserialize)]
pub struct UpdateUserPrefs{
    email: String,
    editorWidth: String,
    editorPosition: String,
}

pub async fn update_user_prefs(
    client: Extension<Client>,
    Json(payload): Json<UpdateUserPrefs>
    ) -> impl IntoResponse {
   
    let users = client.clone().database("notes-app").collection::<User>("users");

    let update = users.update_one(doc! {"email": payload.email}, doc!{ 
        "$set":{"prefs":
        Bson::Document(doc!{"editorWidth": payload.editorWidth,
                            "editorPosition": payload.editorPosition,
                        })
        }}
        , None).await.unwrap();

    Json(update)
}

#[derive(Deserialize)]
pub struct UpdateNote{
    _id: ObjectId,
    title: String,
    text: String,
    folder: String,
    index: i32,
       
}

pub async fn update_note(
    client: Extension<Client>,
    Json(payload): Json<UpdateNote>
    ) -> impl IntoResponse{
    
    let notes = client.clone().database("notes-app").collection::<Note>("notes");

    let update = notes.update_one(doc!{"_id": payload._id}, 
                     doc!{"$set":{"title":payload.title, "text":payload.text, "folder": payload.folder, "index": payload.index}}, None).await.unwrap();

    Json(update)
}

#[derive(Deserialize)]
pub struct UpdateFolder{
    email: String,
    folder_id: String,
    name: String,
    order: i32,
    opened: bool,
}

pub async fn update_user_folder(
    client: Extension<Client>,
    Json(payload): Json<UpdateFolder>
    ) -> impl IntoResponse{
    
    let users = client.clone().database("notes-app").collection::<User>("users");

    let update = users.update_one(doc!{"email":payload.email, "folders._id": &payload.folder_id}, 
                                  doc!{"$set":
                                        {"folders.$":
                                          Bson::Document(doc!{"_id": payload.folder_id,
                                          "name":payload.name,
                                          "order":payload.order,
                                          "opened":payload.opened})
                                        }
                                  }, None).await.unwrap(); 
    Json(update)

}

#[derive(Deserialize)]
pub struct UpdateUserFolders{
    email: String,
    folders: Vec<Folder>,
}
pub async fn update_all_user_folders(
    client: Extension<Client>,
    Json(payload): Json<UpdateUserFolders>
    ) -> impl IntoResponse{

    let users = client.clone().database("notes-app").collection::<User>("users");

    let mut folder_vec: Vec<Bson> = Vec::new();
    
    for folder in payload.folders.iter(){
        //print!("{} | ", folder.name);
        folder_vec.push(
            Bson::Document(doc!{
                "_id": &folder._id,
                "name": &folder.name,
                "order": &folder.order,
                "opened": &folder.opened
            }));
    }

    let update = users.update_one(doc! {"email":payload.email},
        doc!{"$set":{"folders":  folder_vec}}, None).await.unwrap();

    Json(update)
}
