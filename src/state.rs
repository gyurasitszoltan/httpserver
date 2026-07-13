use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{config::Config, mail::Mail};

#[derive(Clone)]
pub(crate) struct StateData {
    pub(crate) db: SqlitePool,
    pub(crate) cfg: Arc<Config>,
    pub(crate) mail: Mail,
}
