use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json,
    Router,
};

use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[derive(Serialize)]
struct PackageResponse {
    name: String,
    latest: String,
    url: String,
    sha256: String,
}

#[tokio::main]
async fn main() {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:registry.db")
        .await
        .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS packages (
            name TEXT PRIMARY KEY,
            description TEXT,
            author TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_name TEXT NOT NULL,
            version TEXT NOT NULL,
            url TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            published_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(package_name, version),
            FOREIGN KEY(package_name) REFERENCES packages(name)
        );
        "#
    )
    .execute(&db)
    .await
    .unwrap();

    let app = Router::new()
        .route("/api/package:name", get(get_package))
        .with_state(AppState { db });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();

}

async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {

    let row = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT
            p.name,
            v.version,
            v.url,
            v.sha256,
        FROM packages p
        JOIN versions v ON v.package_name = p.name
        WHERE p.name = ?
        ORDER BY v.published_at DESC, v.id DESC
        LIMIT 1;
        "#
    )
    .bind(&name)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((name, latest, url, sha256))) => {
            Json(PackageResponse {
                name,
                latest,
                url,
                sha256,
            }).into_response()
        }

        Ok(None) => {
            (StatusCode::NOT_FOUND, "Package not found").into_response()
        }

        Err(err) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            ).into_response()
        }
    }
}
