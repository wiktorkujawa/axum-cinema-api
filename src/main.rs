use anyhow::anyhow;
use axum::{
    extract::{Extension, WebSocketUpgrade},
    http::{header, HeaderValue, Method},
    routing::{delete, get, patch, post},
    Router,
};
use mongodb::{bson::doc, options::ClientOptions, Client};
use tokio::sync::Mutex;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod controllers;
pub mod models;
mod utils;
use controllers::{
    hall_controller::*, home_controller, movie_controller::*, session_controller::*,
};

mod websockets;
use shuttle_runtime::{SecretStore, Secrets};

use crate::websockets::{websocket_handler, SharedState};

#[shuttle_runtime::main]
async fn main(#[Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    // get secret defined in `Secrets.toml` file.
    let database_url = if let Some(secret) = secret_store.get("MONGODB_URI") {
        secret
    } else {
        return Err(anyhow!("secret was not found").into());
    };

    let app_url = if let Some(secret) = secret_store.get("APP_URL") {
        secret
    } else {
        return Err(anyhow!("secret was not found").into());
    };

    // For different deployment than shuttle use dotenv
    // dotenv().ok(); // Load environment variables
    // env::set_var("RUST_LOG", "debug");
    // let database_url = env::var("MONGODB_URI").expect("MONGODB_URI must be set");

    let client_options = ClientOptions::parse(&database_url)
        .await
        .expect("Failed to connect to MongoDB");
    let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");

    // Ping the server to see if you can connect to the cluster
    client
        .database("cinema-axum")
        .run_command(doc! {"ping": 1}, None)
        .await
        .unwrap();
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    let shared_client = Arc::new(client.clone());

    let shared_state = Arc::new(Mutex::new(SharedState::new()));

    let shared_cliented = Arc::new(client.clone());

    let app = Router::new()
        .route("/", get(home_controller::index))
        .route("/ws", get(move |ws: WebSocketUpgrade| websocket_handler(ws, Extension(shared_cliented), shared_state.clone())))
        .route("/sessions", get(load_sessions_with_details))
        .route("/sessions/{id}", get(fetch_session_by_id))
        .route("/movies", get(load_movies_with_details))
        .route("/movies", post(add_movie))
        .route("/movies/{id}", get(load_movie_with_details))
        .route("/movies/{id}", patch(update_movie))
        .route("/movies/{id}", delete(delete_movie))
        .route("/halls", get(load_halls_with_details))
        .route("/halls", post(add_hall))
        .route("/halls/{id}", get(load_hall_with_details))
        .route("/halls/{id}", patch(update_hall))
        .route("/halls/{id}", delete(delete_hall))
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_origin(app_url.parse::<HeaderValue>().unwrap())
                .allow_headers([header::CONTENT_TYPE]),
        )
        .layer(Extension(shared_client.clone()));

    // run our app with hyper, listening globally on port 4000 with Tokio - no shuttle deployment
    // let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    // axum::serve(listener, app).await.unwrap();

    Ok(app.into())
}
