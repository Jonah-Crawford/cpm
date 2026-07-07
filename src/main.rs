// src/main.rs

use serde::Serialize;
use tower_http::services::ServeDir;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
};

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

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct PublishPackageRequest {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
    url: String,
    sha256: String,
}

#[derive(Serialize)]
struct VersionItem {
    version: String,
    url: String,
    sha256: String,
    published_at: String,
}

#[derive(Serialize)]
struct PackageListItem {
    name: String,
    description: Option<String>,
    author: Option<String>,
    latest: Option<String>,
    published_at: Option<String>,
}

async fn list_packages(
    State(state): State<AppState>,
) -> impl IntoResponse {

    let rows = sqlx::query_as::<_, (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>(
        r#"
        SELECT
          p.name,
          p.description,
          p.author,
          v.version,
          v.published_at
        FROM packages p
        LEFT JOIN versions v ON v.id = (
          SELECT id
          FROM versions
          WHERE package_name = p.name
          ORDER BY published_at DESC, id DESC
          LIMIT 1
        )
        ORDER BY p.name ASC;
        "#
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let packages: Vec<PackageListItem> = rows
                .into_iter()
                .map(|(name, description, author, latest, published_at)| {
                     PackageListItem {
                         name,
                         description,
                         author,
                         latest,
                         published_at,
                     }
                })
                .collect();

              Json(packages).into_response()
        }

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", err),
        ).into_response(),
    }
}

async fn list_versions(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {

    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT version, url, sha256, published_at
        FROM versions
        WHERE package_name = ?
        ORDER BY published_at DESC, id DESC;
        "#
    )
    .bind(&name)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) if rows.is_empty() => {
            (StatusCode::NOT_FOUND, "Package not found").into_response()
        }

        Ok(rows) => {
            let versions: Vec<VersionItem> = rows
                .into_iter()
                .map(|(version, url, sha256, published_at)| VersionItem {
                    version,
                    url,
                    sha256,
                    published_at,
                  })
                  .collect();

              Json(versions).into_response()
        }

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", err),
        ).into_response(),
    }
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
            v.sha256
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

async fn publish_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(pkg): Json<PublishPackageRequest>,
) -> Result<StatusCode, (StatusCode, String)> {

    let expected_token = std::env::var("CPM_PUBLISH_TOKEN")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Registry missing CPM_PUBLISH_TOKEN".to_string(),
            )
        })?;

    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if auth != format!("Bearer {}", expected_token) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid publish token".to_string(),
        ));
    }

    if pkg.name.trim().is_empty() || pkg.version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Package name and version are required".to_string(),
        ));
    }

    let mut tx = state.db.begin()
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    sqlx::query(
        r#"
        INSERT INTO packages
          (name, description, author)
        VALUES
          (?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
          description = excluded.description,
          author = excluded.author;
        "#
    )
    .bind(&pkg.name)
    .bind(&pkg.description)
    .bind(&pkg.author)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = sqlx::query(
        r#"
            INSERT INTO versions
              (package_name, version, url, sha256)
            VALUES
              (?, ?, ?, ?);
        "#
    )
    .bind(&pkg.name)
    .bind(&pkg.version)
    .bind(&pkg.url)
    .bind(&pkg.sha256)
    .execute(&mut *tx)
    .await;

    match result {
        Ok(_) => {
            tx.commit().await.map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

            Ok(StatusCode::CREATED)
        }

        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err((
                StatusCode::CONFLICT,
                format!("{} v{} already exists", pkg.name, pkg.version),
            ))
        }

        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
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
        .route("/api", get(|| async { "CPM registry online" }))
        .route("/api/packages", get(list_packages))
        .route("/api/package", post(publish_package))
        .route("/api/package/:name/versions", get(list_versions))
        .route("/api/package/:name", get(get_package))
        .nest_service("/", ServeDir::new("public"))
        .with_state(AppState { db });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();

}
