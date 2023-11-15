use axum::{
    routing::get,
    response::IntoResponse,
    Extension,
    Router,
    Json
};
use std::fmt;
use futures::stream::StreamExt;
use dotenv::dotenv;
//use hyper::{Response, header::Values};
//use hyper::header::Values;
use serde::{Serialize, Deserialize};

use mongodb::{
    bson::doc,
    bson::oid::ObjectId,
    Client,
    //Collection
};

#[derive(Serialize, Deserialize)]
struct Note {
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

/* 
     * Returns ONE document from MongoDB
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set.");
        let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to Establish MongoDB Connection");

    let app = Router::new().route("/", get(hello_world))
        .route("/get_all_notes", get(get_all_notes)).layer(Extension(client));
    let server_port = std::env::var("PORT").expect("PORT must be set.");
    axum::Server::bind(&("0.0.0.0:".to_owned()+&server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();


}
