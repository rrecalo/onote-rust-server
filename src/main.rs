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
    let v: Vec<Note> = cursor.expect("REASON")
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
    let delete_from_user = users.find_one_and_update(doc! {"email": payload.email},
                                doc! {"$pull" : {"notes": payload._id.to_string()}}, None).await;
    //println!("{}", delete_from_user.unwrap().unwrap());
    Json(delete_result.unwrap())
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
        .route("/delete_user_note", delete(delete_user_note))
        .layer(Extension(client));
    let server_port = std::env::var("PORT").expect("PORT must be set.");
    //serve locally on server_port from .env file
    axum::Server::bind(&("0.0.0.0:".to_owned()+&server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();


}
