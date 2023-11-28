use axum::{
    routing::{get, put, post, delete},
    Extension,
    Router,
};
use dotenv::dotenv;

use mongodb::Client;

use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

mod model;
mod handlers;
use crate::handlers::handlers::*;

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


#[tokio::main]
async fn main() {
    dotenv().ok();
   
    //setup database connection
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set.");
        let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to Establish MongoDB Connection");

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    //add app routes
    let app = Router::new().route("/", get(hello_world))
        .route("/get_all_notes", get(get_all_notes))
        .route("/get_user_by_email/:user_email", get(get_user_by_email))
        .route("/get_notes_by_user/:user_email", get(get_notes_by_user))
        .route("/create_user_note", post(create_user_note))
        .route("/create_user_folder", put(create_user_folder))
        .route("/delete_user_note", delete(delete_user_note))
        .route("/delete_user_folder", put(delete_user_folder))
        .route("/update_user_last_note", put(update_user_last_note))
        .route("/update_user_prefs", put(update_user_prefs))
        .route("/update_note", put(update_note))
        .route("/update_user_folder", put(update_user_folder))
        .route("/update_all_user_folders", put(update_all_user_folders))
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .into_inner(),
        )
        .layer(Extension(client));
    let server_port = std::env::var("PORT").expect("PORT must be set.");
    //serve locally on server_port from .env file
    axum::Server::bind(&("0.0.0.0:".to_owned()+&server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();


}
