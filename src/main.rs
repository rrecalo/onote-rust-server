use axum::{
    routing::{get, put, post, delete},
    extract::Path,
    Extension,
    Router,
    response::sse::{Event, Sse, KeepAlive},

    
};
use dotenv::dotenv;

use mongodb::Client;

use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

mod model;
mod handlers;
use crate::handlers::handlers::*;
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;

async fn sse_handler(
    client: Extension<Client>,
    Path(note_id): Path<String>
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A `Stream` that repeats an event every second
    let stream = stream::unfold((), move |()| {  
        let c = client.clone();
        let _id = note_id.clone();
        async move { Some((get_note_contents(c, _id).await, ())) }
        })
        .map(Ok)
        .throttle(Duration::from_millis(100));

    Sse::new(stream).keep_alive(
        //KeepAlive::default()
        axum::response::sse::KeepAlive::new().
        interval(Duration::from_secs(1))
        .text("keep-alive-text"),
        )
}

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
        .route("/collaborate/:note_id", get(sse_handler))
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
