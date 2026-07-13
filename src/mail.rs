use postmark::{
    Query as PostmarkQuery,
    api::{Body, email::SendEmailRequest},
    reqwest::PostmarkClient,
};
use tracing::{error, info};

use crate::{
    config::Config,
    error::{E, R},
};

#[derive(Clone)]
pub(crate) struct Mail {
    client: PostmarkClient,
    from: String,
    stream: String,
}

impl Mail {
    pub(crate) fn new(config: &Config) -> Self {
        Self {
            client: PostmarkClient::builder()
                .server_token(&config.token)
                .build(),
            from: config.from.clone(),
            stream: config.stream.clone(),
        }
    }

    pub(crate) async fn send(&self, to: &str, url: &str) -> R<()> {
        let request = SendEmailRequest::builder()
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
        let response = request
            .execute(&self.client)
            .await
            .map_err(|error_value| {
                error!(%error_value, "postmark");
                E::Internal("Az e-mail küldése sikertelen".into())
            })?
            .error_for_status()
            .map_err(|response| {
                error!(code = response.error_code, message = %response.message, "postmark");
                E::Internal("Az e-mail küldése sikertelen".into())
            })?;
        info!(message_id = ?response.message_id, "e-mail elküldve");
        Ok(())
    }
}
