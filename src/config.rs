use std::{env, net::SocketAddr};

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) addr: SocketAddr,
    pub(crate) db: String,
    pub(crate) base: String,
    pub(crate) cors: String,
    pub(crate) dist: String,
    pub(crate) cookie: String,
    pub(crate) secure: bool,
    pub(crate) session_hours: i64,
    pub(crate) ttl: i64,
    pub(crate) per_hour: i64,
    pub(crate) token: String,
    pub(crate) from: String,
    pub(crate) stream: String,
    pub(crate) admin: String,
}

impl Config {
    pub(crate) fn load() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        let value = |key: &str, default: &str| env::var(key).unwrap_or_else(|_| default.into());
        let required =
            |key: &str| env::var(key).map_err(|_| format!("Hiányzó környezeti változó: {key}"));
        let number = |key: &str, default: i64| {
            value(key, &default.to_string())
                .parse()
                .map_err(|_| format!("Érvénytelen {key}"))
        };
        let base = value("APP_BASE_URL", "http://localhost:3000")
            .trim_end_matches('/')
            .to_string();
        url::Url::parse(&base).map_err(|_| "Érvénytelen APP_BASE_URL".to_string())?;

        Ok(Self {
            addr: value("BIND_ADDR", "127.0.0.1:3000")
                .parse()
                .map_err(|_| "Érvénytelen BIND_ADDR")?,
            db: value("DATABASE_URL", "sqlite://./data/app.db?mode=rwc"),
            base,
            cors: value("CORS_ORIGIN", "http://localhost:5173"),
            dist: value("FRONTEND_DIST_DIR", "./frontend/dist"),
            cookie: value("SESSION_COOKIE_NAME", "app_session"),
            secure: value("SESSION_SECURE", "false")
                .parse()
                .map_err(|_| "Érvénytelen SESSION_SECURE")?,
            session_hours: number("SESSION_MAX_AGE_HOURS", 168)?,
            ttl: number("MAGIC_LINK_TTL_MINUTES", 15)?,
            per_hour: number("MAGIC_LINK_REQUESTS_PER_HOUR", 5)?,
            token: required("POSTMARK_SERVER_TOKEN")?,
            from: required("POSTMARK_FROM")?,
            stream: value("POSTMARK_MESSAGE_STREAM", "outbound"),
            admin: required("BOOTSTRAP_ADMIN_EMAIL")?,
        })
    }
}
