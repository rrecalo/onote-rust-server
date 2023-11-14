use axum::{
    routing::get,
    Router,
};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new().route("/", get(|| async {"Hello, World!" }));
    let server_port = std::env::var("PORT").expect("PORT must be set.");
    axum::Server::bind(&("0.0.0.0:".to_owned()+&server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();


}
