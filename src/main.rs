use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, AuthSession, AuthUser, AuthnBackend, UserId};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use postmark::{
    Query as PostmarkQuery,
    api::{Body, email::SendEmailRequest},
    reqwest::PostmarkClient,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    FromRow, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use thiserror::Error;
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
use tracing::{error, info};
const COLUMNS: &str =
    "id,email,display_name,role,is_active,session_version,created_at,last_login_at";
type R<T> = std::result::Result<T, E>;
type Session = AuthSession<Backend>;
#[derive(Clone)]
struct Config {
    addr: SocketAddr,
    db: String,
    base: String,
    cors: String,
    dist: String,
    cookie: String,
    secure: bool,
    session_hours: i64,
    ttl: i64,
    per_hour: i64,
    token: String,
    from: String,
    stream: String,
    admin: String,
}
impl Config {
    fn load() -> std::result::Result<Self, String> {
        dotenvy::dotenv().ok();
        let v = |k: &str, d: &str| env::var(k).unwrap_or_else(|_| d.into());
        let req = |k: &str| env::var(k).map_err(|_| format!("Hiányzó környezeti változó: {k}"));
        let n = |k: &str, d: i64| {
            v(k, &d.to_string())
                .parse()
                .map_err(|_| format!("Érvénytelen {k}"))
        };
        let base = v("APP_BASE_URL", "http://localhost:3000")
            .trim_end_matches('/')
            .to_string();
        url::Url::parse(&base).map_err(|_| "Érvénytelen APP_BASE_URL".to_string())?;
        Ok(Self {
            addr: v("BIND_ADDR", "127.0.0.1:3000")
                .parse()
                .map_err(|_| "Érvénytelen BIND_ADDR")?,
            db: v("DATABASE_URL", "sqlite://./data/app.db?mode=rwc"),
            base,
            cors: v("CORS_ORIGIN", "http://localhost:5173"),
            dist: v("FRONTEND_DIST_DIR", "./frontend/dist"),
            cookie: v("SESSION_COOKIE_NAME", "app_session"),
            secure: v("SESSION_SECURE", "false")
                .parse()
                .map_err(|_| "Érvénytelen SESSION_SECURE")?,
            session_hours: n("SESSION_MAX_AGE_HOURS", 168)?,
            ttl: n("MAGIC_LINK_TTL_MINUTES", 15)?,
            per_hour: n("MAGIC_LINK_REQUESTS_PER_HOUR", 5)?,
            token: req("POSTMARK_SERVER_TOKEN")?,
            from: req("POSTMARK_FROM")?,
            stream: v("POSTMARK_MESSAGE_STREAM", "outbound"),
            admin: req("BOOTSTRAP_ADMIN_EMAIL")?,
        })
    }
}
#[derive(Clone)]
struct StateData {
    db: SqlitePool,
    cfg: Arc<Config>,
    mail: Mail,
}
#[derive(Debug, Error)]
enum E {
    #[error("{0}")]
    Bad(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error("rate limit")]
    Rate,
    #[error("{0}")]
    Internal(String),
}
impl From<sqlx::Error> for E {
    fn from(x: sqlx::Error) -> Self {
        error!(%x,"sqlx");
        Self::Internal("Adatbázis hiba".into())
    }
}
impl IntoResponse for E {
    fn into_response(self) -> Response {
        let s = match self {
            Self::Bad(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Rate => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let m = match &self {
            Self::Bad(x) | Self::Conflict(x) => x.clone(),
            Self::Unauthorized => "Bejelentkezés szükséges.".into(),
            Self::Forbidden => "Nincs jogosultságod ehhez a művelethez.".into(),
            Self::NotFound => "A kért erőforrás nem található.".into(),
            Self::Rate => "Túl sok kérés. Próbáld később.".into(),
            Self::Internal(x) => {
                error!(%x,"api error");
                "Váratlan szerverhiba történt.".into()
            }
        };
        (
            s,
            Json(serde_json::json!({"error":{"code":s.as_str(),"message":m}})),
        )
            .into_response()
    }
}
#[derive(Clone, Debug, FromRow)]
struct User {
    id: i64,
    email: String,
    display_name: Option<String>,
    role: String,
    is_active: i64,
    session_version: String,
    created_at: i64,
    last_login_at: Option<i64>,
}
impl AuthUser for User {
    type Id = i64;
    fn id(&self) -> i64 {
        self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
        self.session_version.as_bytes()
    }
}
#[derive(Clone)]
struct Backend {
    db: SqlitePool,
}
#[derive(Clone)]
struct Cred {
    i: i64,
}
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Cred;
    type Error = sqlx::Error;
    async fn authenticate(&self, c: Cred) -> std::result::Result<Option<User>, sqlx::Error> {
        active(&self.db, c.i).await
    }
    async fn get_user(&self, i: &UserId<Self>) -> std::result::Result<Option<User>, sqlx::Error> {
        active(&self.db, *i).await
    }
}
async fn active(db: &SqlitePool, id: i64) -> std::result::Result<Option<User>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE id=? AND is_active=1"
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}
#[derive(Clone)]
struct Mail {
    c: PostmarkClient,
    from: String,
    stream: String,
}
impl Mail {
    fn new(x: &Config) -> Self {
        Self {
            c: PostmarkClient::builder().server_token(&x.token).build(),
            from: x.from.clone(),
            stream: x.stream.clone(),
        }
    }
    async fn send(&self, to: &str, url: &str) -> R<()> {
        let req = SendEmailRequest::builder()
            .from(&self.from)
            .to(to)
            .subject("Belépési link")
            .tag("magic-link")
            .body(Body::html_and_text(
                format!("<p><a href=\"{url}\">Biztonságos belépés</a></p>"),
                format!("Belépés: {url}"),
            ))
            .message_stream(&self.stream)
            .build();
        let r = req
            .execute(&self.c)
            .await
            .map_err(|e| {
                error!(%e,"postmark");
                E::Internal("Az e-mail küldése sikertelen".into())
            })?
            .error_for_status()
            .map_err(|r| {
                error!(code=r.error_code,message=%r.message,"postmark");
                E::Internal("Az e-mail küldése sikertelen".into())
            })?;
        info!(message_id=?r.message_id,"e-mail elküldve");
        Ok(())
    }
}
#[derive(Deserialize)]
struct LinkReq {
    email: String,
}
#[derive(Deserialize)]
struct Verify {
    token: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Me {
    id: i64,
    email: String,
    display_name: Option<String>,
    role: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserOut {
    id: i64,
    email: String,
    display_name: Option<String>,
    role: String,
    is_active: bool,
    created_at: i64,
    last_login_at: Option<i64>,
}
impl From<User> for UserOut {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            role: u.role,
            is_active: u.is_active != 0,
            created_at: u.created_at,
            last_login_at: u.last_login_at,
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserList {
    items: Vec<UserOut>,
    page: i64,
    page_size: i64,
    total: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Create {
    email: String,
    display_name: Option<String>,
    role: String,
    is_active: Option<bool>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Update {
    display_name: Option<Option<String>>,
    role: Option<String>,
    is_active: Option<bool>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQ {
    #[serde(default = "one")]
    page: i64,
    #[serde(default = "size")]
    page_size: i64,
    #[serde(default)]
    query: String,
}
fn one() -> i64 {
    1
}
fn size() -> i64 {
    25
}
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let c = Config::load().map_err(std::io::Error::other)?;
    if let Some(p) =
        c.db.strip_prefix("sqlite://")
            .and_then(|x| x.split('?').next())
            .and_then(|x| std::path::Path::new(x).parent())
    {
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(p)?
        }
    }
    let o = SqliteConnectOptions::from_str(&c.db)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let db = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(o)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    bootstrap(&db, &c.admin).await?;
    let store = SqliteStore::new(db.clone());
    store.migrate().await?;
    let sessions = SessionManagerLayer::new(store)
        .with_name(c.cookie.clone())
        .with_http_only(true)
        .with_secure(c.secure)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(c.session_hours)));
    let auth = AuthManagerLayerBuilder::new(Backend { db: db.clone() }, sessions).build();
    let state = StateData {
        db: db.clone(),
        cfg: Arc::new(c.clone()),
        mail: Mail::new(&c),
    };
    let api = Router::new()
        .route("/healthz", get(health))
        .route("/api/auth/magic-link", post(link))
        .route("/api/auth/verify-magic-link", post(verify))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/admin/users", get(list).post(create))
        .route(
            "/api/admin/users/{id}",
            get(show).patch(update).delete(remove),
        )
        .with_state(state);
    let app = api
        .fallback_service(
            ServeDir::new(&c.dist)
                .not_found_service(ServeFile::new(format!("{}/index.html", c.dist))),
        )
        .layer(auth)
        .layer(RequestBodyLimitLayer::new(16384))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(15),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(c.cors.parse::<header::HeaderValue>()?)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http());
    let l = tokio::net::TcpListener::bind(c.addr).await?;
    info!(address=%c.addr,"szerver elindult");
    axum::serve(l, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
async fn bootstrap(db: &SqlitePool, email: &str) -> R<()> {
    let t = now();
    sqlx::query("INSERT INTO users (email,role,is_active,session_version,created_at,updated_at) SELECT ?, 'admin',1,?,?,? WHERE NOT EXISTS (SELECT 1 FROM users WHERE role='admin' AND is_active=1)").bind(email_of(email)?).bind(token()).bind(t).bind(t).execute(db).await?;
    Ok(())
}
async fn health(State(s): State<StateData>) -> R<StatusCode> {
    sqlx::query("SELECT 1").execute(&s.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn link(State(s): State<StateData>, Json(x): Json<LinkReq>) -> R<StatusCode> {
    let email = email_of(&x.email)?;
    let u: Option<User> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE email=? AND is_active=1"
    ))
    .bind(email)
    .fetch_optional(&s.db)
    .await?;
    let Some(u) = u else {
        return Ok(StatusCode::ACCEPTED);
    };
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM magic_link_tokens WHERE user_id=? AND created_at>=?",
    )
    .bind(u.id)
    .bind(now() - 3600)
    .fetch_one(&s.db)
    .await?;
    if n >= s.cfg.per_hour {
        return Err(E::Rate);
    }
    let raw = token();
    let t = now();
    sqlx::query("UPDATE magic_link_tokens SET used_at=? WHERE user_id=? AND used_at IS NULL")
        .bind(t)
        .bind(u.id)
        .execute(&s.db)
        .await?;
    sqlx::query(
        "INSERT INTO magic_link_tokens (user_id,token_hash,expires_at,created_at) VALUES (?,?,?,?)",
    )
    .bind(u.id)
    .bind(hash(&raw))
    .bind(t + s.cfg.ttl * 60)
    .bind(t)
    .execute(&s.db)
    .await?;
    s.mail
        .send(
            &u.email,
            &format!("{}/auth/callback?token={raw}", s.cfg.base),
        )
        .await?;
    Ok(StatusCode::ACCEPTED)
}
async fn verify(
    State(s): State<StateData>,
    mut session: Session,
    Json(x): Json<Verify>,
) -> R<Json<Me>> {
    if x.token.len() < 32 {
        return Err(E::Unauthorized);
    }
    let mut tx = s.db.begin().await?;
    let row:Option<(i64,i64)>=sqlx::query_as("SELECT id,user_id FROM magic_link_tokens WHERE token_hash=? AND used_at IS NULL AND expires_at>?").bind(hash(&x.token)).bind(now()).fetch_optional(&mut *tx).await?;
    let Some((tid, uid)) = row else {
        return Err(E::Unauthorized);
    };
    let u: Option<User> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE id=? AND is_active=1"
    ))
    .bind(uid)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(u) = u else {
        return Err(E::Unauthorized);
    };
    if sqlx::query("UPDATE magic_link_tokens SET used_at=? WHERE id=? AND used_at IS NULL")
        .bind(now())
        .bind(tid)
        .execute(&mut *tx)
        .await?
        .rows_affected()
        != 1
    {
        return Err(E::Unauthorized);
    }
    sqlx::query("UPDATE users SET last_login_at=?,updated_at=? WHERE id=?")
        .bind(now())
        .bind(now())
        .bind(u.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    session.login(&u).await.map_err(|e| {
        error!(%e,"session");
        E::Internal("Session létrehozása sikertelen".into())
    })?;
    Ok(Json(me_of(&u)))
}
async fn logout(mut s: Session) -> R<StatusCode> {
    s.logout().await.map_err(|e| E::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
async fn me(s: Session) -> R<Json<Me>> {
    Ok(Json(me_of(&user(&s)?)))
}
async fn list(State(s): State<StateData>, a: Session, Query(q): Query<ListQ>) -> R<Json<UserList>> {
    admin(&a)?;
    let p = q.page.max(1);
    let z = q.page_size.clamp(1, 100);
    let x = format!("%{}%", q.query.trim());
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email LIKE ? OR COALESCE(display_name,'') LIKE ?",
    )
    .bind(&x)
    .bind(&x)
    .fetch_one(&s.db)
    .await?;
    let v:Vec<User>=sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE email LIKE ? OR COALESCE(display_name,'') LIKE ? ORDER BY id DESC LIMIT ? OFFSET ?")).bind(&x).bind(&x).bind(z).bind((p-1)*z).fetch_all(&s.db).await?;
    Ok(Json(UserList {
        items: v.into_iter().map(Into::into).collect(),
        page: p,
        page_size: z,
        total,
    }))
}
async fn show(State(s): State<StateData>, a: Session, Path(id): Path<i64>) -> R<Json<UserOut>> {
    admin(&a)?;
    Ok(Json(find(&s.db, id).await?.ok_or(E::NotFound)?.into()))
}
async fn create(
    State(s): State<StateData>,
    a: Session,
    Json(x): Json<Create>,
) -> R<(StatusCode, Json<UserOut>)> {
    admin(&a)?;
    let t = now();
    let r=sqlx::query("INSERT INTO users (email,display_name,role,is_active,session_version,created_at,updated_at) VALUES (?,?,?,?,?,?,?)").bind(email_of(&x.email)?).bind(name(x.display_name)?).bind(role(&x.role)?).bind(x.is_active.unwrap_or(true)as i64).bind(token()).bind(t).bind(t).execute(&s.db).await;
    match r {
        Ok(r) => Ok((
            StatusCode::CREATED,
            Json(find(&s.db, r.last_insert_rowid()).await?.unwrap().into()),
        )),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            Err(E::Conflict("Ez az e-mail cím már használatban van.".into()))
        }
        Err(e) => Err(e.into()),
    }
}
async fn update(
    State(s): State<StateData>,
    a: Session,
    Path(id): Path<i64>,
    Json(x): Json<Update>,
) -> R<Json<UserOut>> {
    let act = admin(&a)?;
    let old = find(&s.db, id).await?.ok_or(E::NotFound)?;
    let r = match x.role {
        Some(v) => role(&v)?,
        None => old.role.clone(),
    };
    let on = x.is_active.unwrap_or(old.is_active != 0);
    if act.id == id && !on {
        return Err(E::Conflict("Saját fiók nem deaktiválható.".into()));
    }
    if old.role == "admin" && old.is_active != 0 && (r != "admin" || !on) {
        last(&s.db).await?
    }
    let n = match x.display_name {
        Some(v) => name(v)?,
        None => old.display_name.clone(),
    };
    let ver = if r != old.role || on != (old.is_active != 0) {
        token()
    } else {
        old.session_version
    };
    sqlx::query("UPDATE users SET display_name=?,role=?,is_active=?,session_version=?,updated_at=? WHERE id=?").bind(n).bind(r).bind(on as i64).bind(ver).bind(now()).bind(id).execute(&s.db).await?;
    Ok(Json(find(&s.db, id).await?.unwrap().into()))
}
async fn remove(State(s): State<StateData>, a: Session, Path(id): Path<i64>) -> R<StatusCode> {
    let act = admin(&a)?;
    let u = find(&s.db, id).await?.ok_or(E::NotFound)?;
    if act.id == id {
        return Err(E::Conflict("Saját fiók nem deaktiválható.".into()));
    }
    if u.role == "admin" && u.is_active != 0 {
        last(&s.db).await?
    }
    sqlx::query("UPDATE users SET is_active=0,session_version=?,updated_at=? WHERE id=?")
        .bind(token())
        .bind(now())
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
fn user(s: &Session) -> R<User> {
    s.user.clone().ok_or(E::Unauthorized)
}
fn admin(s: &Session) -> R<User> {
    let u = user(s)?;
    if u.role == "admin" && u.is_active != 0 {
        Ok(u)
    } else {
        Err(E::Forbidden)
    }
}
async fn find(db: &SqlitePool, id: i64) -> R<Option<User>> {
    Ok(
        sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE id=?"))
            .bind(id)
            .fetch_optional(db)
            .await?,
    )
}
async fn last(db: &SqlitePool) -> R<()> {
    let n: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role='admin' AND is_active=1")
            .fetch_one(db)
            .await?;
    if n <= 1 {
        Err(E::Conflict(
            "Az utolsó aktív admin nem módosítható vagy deaktiválható.".into(),
        ))
    } else {
        Ok(())
    }
}
fn me_of(u: &User) -> Me {
    Me {
        id: u.id,
        email: u.email.clone(),
        display_name: u.display_name.clone(),
        role: u.role.clone(),
    }
}
fn email_of(x: &str) -> R<String> {
    let x = x.trim().to_lowercase();
    if x.len() > 254 || !x.contains('@') || x.starts_with('@') || x.ends_with('@') {
        Err(E::Bad("Érvényes e-mail cím szükséges.".into()))
    } else {
        Ok(x)
    }
}
fn role(x: &str) -> R<String> {
    match x {
        "admin" | "user" => Ok(x.into()),
        _ => Err(E::Bad("A role értéke admin vagy user lehet.".into())),
    }
}
fn name(x: Option<String>) -> R<Option<String>> {
    let x = x
        .map(|x| x.trim().into())
        .filter(|x: &String| !x.is_empty());
    if x.as_ref().is_some_and(|x| x.len() > 120) {
        Err(E::Bad("A név legfeljebb 120 karakter lehet.".into()))
    } else {
        Ok(x)
    }
}
fn now() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}
fn token() -> String {
    let mut b = [0; 32];
    rand::rng().fill_bytes(&mut b);
    URL_SAFE_NO_PAD.encode(b)
}
fn hash(x: &str) -> String {
    format!("{:x}", Sha256::digest(x.as_bytes()))
}
