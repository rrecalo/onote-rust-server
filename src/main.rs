use std::str::FromStr;

use axum::{
    routing::get,
    response::IntoResponse,
    Extension,
    Router,
    extract::Path,
    Json
};
use futures::stream::StreamExt;
use dotenv::dotenv;
//use hyper::{Response, header::Values};
//use hyper::header::Values;

use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
    //Collection
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
async fn get_notes_by_user(
    client: Extension<Client>,
    Path(user_email): Path<String>
    ) -> impl IntoResponse {

    //get the user object to receive its 'notes' array
    let user = get_user(client.clone(), axum::extract::Path(user_email)).await.unwrap();
    
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
        .route("/get_notes_by_user/:user_email", get(get_notes_by_user)).layer(Extension(client));
    let server_port = std::env::var("PORT").expect("PORT must be set.");
    //serve locally on server_port from .env file
    axum::Server::bind(&("0.0.0.0:".to_owned()+&server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();


}
