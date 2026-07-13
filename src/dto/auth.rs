use serde::{Deserialize, Serialize};

use crate::auth::User;

#[derive(Deserialize)]
pub(crate) struct LinkReq {
    pub(crate) email: String,
}

#[derive(Deserialize)]
pub(crate) struct Verify {
    pub(crate) token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Me {
    id: i64,
    email: String,
    display_name: Option<String>,
    role: String,
}

impl From<&User> for Me {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            role: user.role.clone(),
        }
    }
}
