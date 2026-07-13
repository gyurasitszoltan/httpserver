use std::{str::FromStr, sync::Arc};

use axum::{
    Router,
    http::{Method, StatusCode, header},
};
use axum_login::AuthManagerLayerBuilder;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use time::Duration;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

use crate::{auth::Backend, config::Config, db::users, mail::Mail, routes, state::StateData};

pub(crate) async fn build(config: &Config) -> Result<Router, Box<dyn std::error::Error>> {
    create_database_directory(&config.db)?;
    let options = SqliteConnectOptions::from_str(&config.db)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let db = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    users::bootstrap(&db, &config.admin).await?;

    let store = SqliteStore::new(db.clone());
    store.migrate().await?;
    let sessions = SessionManagerLayer::new(store)
        .with_name(config.cookie.clone())
        .with_http_only(true)
        .with_secure(config.secure)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(config.session_hours)));
    let auth = AuthManagerLayerBuilder::new(Backend::new(db.clone()), sessions).build();
    let state = StateData {
        db,
        cfg: Arc::new(config.clone()),
        mail: Mail::new(config),
    };
    let api = routes::api_router(state);

    Ok(api
        .fallback_service(
            ServeDir::new(&config.dist)
                .not_found_service(ServeFile::new(format!("{}/index.html", config.dist))),
        )
        .layer(auth)
        .layer(RequestBodyLimitLayer::new(16_384))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(15),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(config.cors.parse::<header::HeaderValue>()?)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http()))
}

fn create_database_directory(database_url: &str) -> std::io::Result<()> {
    if let Some(parent) = database_url
        .strip_prefix("sqlite://")
        .and_then(|value| value.split('?').next())
        .and_then(|value| std::path::Path::new(value).parent())
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
