use std::str::FromStr;
use std::fmt;
use rand::Rng;
use serde::Deserialize;
use axum::{
    routing::{get, put, delete},
    response::IntoResponse,
    Extension,
    Router,
    extract::Path,
    Json
};
use futures::stream::StreamExt;
use dotenv::dotenv;

use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    Client,
};

mod model;
use crate::model::{note::Note, user::User};

/* This code block returns ONE document from MongoDB
    let notes = match collection.find_one(doc! {}, none).await{
        ok(some(document)) => ok(json(document)),
        ok(none) => {
            println!("nothing found");
            return err(())
        }
        err(e)=>{
            dbg!(&e);
            return err(())

        }
    };
    return note;
    */

async fn hello_world() -> String {
    
    String::from("Hello World!")
}

async fn get_all_notes(
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

async fn get_user(
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

async fn get_user_by_email(
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

async fn get_notes_by_user(
    client: Extension<Client>,
    Path(user_email): Path<String>
    ) -> impl IntoResponse {

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
   


    /*
     * Debug
    for item in v.iter() {
        println!("{}", item);
    }
    */

    Json(v)

    }

#[derive(Debug, serde::Serialize, Deserialize)]
struct CreateNote{
    email: String,
    note_title: String,
}

impl std::fmt::Display for CreateNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "{0} | {1}", self.email, self.note_title)
    }
}

fn random_note_name() -> String {
        let mut rng = rand::thread_rng();
        let nums: Vec<u8> = (0..4).map(|_| rng.gen_range(0..=9)).collect();
        let result = format!("NewNote{0}{1}{2}{3}", nums[0], nums[1], nums[2], nums[3]);

        result
    }

async fn create_user_note(
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
    inserted
}

#[derive(Deserialize)]
struct DeleteNote{
    email: String,
    _id: ObjectId
}

async fn delete_user_note(
    client: Extension<Client>,
    Json(payload): Json<DeleteNote>
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
struct CreateFolder{
    email: String,
    folder_index: i32,
    folder_name: String,
}

async fn create_user_folder(
    client: Extension<Client>,
    Json(payload): Json<CreateFolder>
    ) -> impl IntoResponse{

    let users = client.clone().database("notes-app").collection::<User>("users");

    let folder_update = users.update_one(doc! {"email": &payload.email},
                            doc! {"$push": {"folders":
                            Bson::Document(doc!{
                                "_id": ObjectId::new().to_string(),
                                "name": payload.folder_name,
                                "order": payload.folder_index,
                                "opened": true,
                            })
                        }}, None).await.unwrap();
    Json(folder_update)
}


#[derive(Deserialize)]
struct DeleteFolder{
    email: String,
    _id: ObjectId
}

async fn delete_user_folder(
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
struct UpdateUserLastNote {
    email: String,
    lastNote: String,
}

async fn update_user_last_note(
    client: Extension<Client>,
    Json(payload): Json<UpdateUserLastNote>
    ) -> impl IntoResponse {
    
    let users = client.clone().database("notes-app").collection::<User>("users");
    let update = users.update_one(doc! {"email": &payload.email}, doc! {"$set":{"lastNote": &payload.lastNote}}, None).await.unwrap();

    //println!("Update {0} {1}'s lastNote to {2}", update.modified_count, payload.email, payload.lastNote);

    Json(update)
}

#[derive(Deserialize)]
struct UpdateUserPrefs{
    email: String,
    editorWidth: String,
    editorPosition: String,
}

async fn update_user_prefs(
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

#[tokio::main]
async fn main() {
    dotenv().ok();
   
    //setup database connection
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set.");
        let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to Establish MongoDB Connection");

    //add app routes
    let app = Router::new().route("/", get(hello_world))
        .route("/get_all_notes", get(get_all_notes))
        .route("/get_user_by_email/:user_email", get(get_user_by_email))
        .route("/get_notes_by_user/:user_email", get(get_notes_by_user))
        .route("/create_user_note", put(create_user_note))
        .route("/create_user_folder", put(create_user_folder))
        .route("/delete_user_note", delete(delete_user_note))
        .route("/delete_user_folder", put(delete_user_folder))
        .route("/update_user_last_note", put(update_user_last_note))
        .route("/update_user_prefs", put(update_user_prefs))
        .layer(Extension(client));
    let server_port = std::env::var("PORT").expect("PORT must be set.");
    //serve locally on server_port from .env file
    axum::Server::bind(&("0.0.0.0:".to_owned()+&server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();


}
