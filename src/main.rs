use axum::{Router, routing::{get, post, delete}};
use crate::handlers::{get_offers, create_offers, cleanup_data};
use std::net::SocketAddr;

mod db;
mod models;
mod handlers;

#[tokio::main]
async fn main() {
    // Initialize the database
    let db = db::init_db().expect("Failed to initialize database");

    // Build our application with some routes
    let app = Router::new()
        .route("/api/offers", get(get_offers).post(create_offers).delete(cleanup_data))
        // Add the database to the app's state
        .with_state(db);

    // Run it with hyper on localhost:8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}