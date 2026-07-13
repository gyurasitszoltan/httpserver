use serde::{Deserialize, Serialize};

use crate::auth::User;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserOut {
    id: i64,
    email: String,
    display_name: Option<String>,
    role: String,
    is_active: bool,
    created_at: i64,
    last_login_at: Option<i64>,
}

impl From<User> for UserOut {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
            is_active: user.is_active != 0,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserList {
    pub(crate) items: Vec<UserOut>,
    pub(crate) page: i64,
    pub(crate) page_size: i64,
    pub(crate) total: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Create {
    pub(crate) email: String,
    pub(crate) display_name: Option<String>,
    pub(crate) role: String,
    pub(crate) is_active: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Update {
    pub(crate) display_name: Option<Option<String>>,
    pub(crate) role: Option<String>,
    pub(crate) is_active: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListQ {
    #[serde(default = "one")]
    pub(crate) page: i64,
    #[serde(default = "size")]
    pub(crate) page_size: i64,
    #[serde(default)]
    pub(crate) query: String,
}

fn one() -> i64 {
    1
}

fn size() -> i64 {
    25
}
