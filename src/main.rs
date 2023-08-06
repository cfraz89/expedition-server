#![feature(once_cell)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

static DB: OnceLock<Surreal<Client>> = OnceLock::new();

enum AppError {
    Status(StatusCode, &'static str),
    Surreal(surrealdb::Error),
    Any(color_eyre::eyre::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Status(s, e) => (s, e).into_response(),
            AppError::Any(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
            }
            AppError::Surreal(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Surreal error: {}", e),
            )
                .into_response(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<color_eyre::eyre::Error>,
{
    fn from(err: E) -> Self {
        AppError::Any(err.into())
    }
}

fn get_db() -> Result<&'static Surreal<Client>, AppError> {
    match DB.get() {
        None => Err(AppError::Status(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get db",
        )),
        Some(db) => Ok(db),
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // initialize tracing
    tracing_subscriber::fmt::init();
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.use_ns("ausadv").use_db("ausadv").await?;
    DB.set(db).unwrap();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/rides", get(get_rides));

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Ride {
    name: String,
}

async fn get_rides() -> Result<Json<Vec<Ride>>, AppError> {
    let rides = get_db()?.select("ride").await?;
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    Ok(Json(rides))
}
