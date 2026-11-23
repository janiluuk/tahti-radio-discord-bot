use std::time::Duration;

use serde::Deserialize;

/// Delays between credential fetch attempts. The bot starts beside the API on
/// every deploy, so the first requests can land while the API is still
/// warming up; about two minutes covers a slow API start before the env
/// fallback is used.
const API_RETRY_DELAYS: [Duration; 8] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
    Duration::from_secs(15),
    Duration::from_secs(30),
    Duration::from_secs(30),
    Duration::from_secs(30),
];

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Env(#[from] envy::Error),

    #[error("{0} is required")]
    Missing(&'static str),

    #[error("invalid Discord application ID")]
    InvalidClientId,
}

#[derive(Deserialize)]
struct EnvConfig {
    discord_token: Option<String>,
    discord_client_id: Option<String>,
    tahti_api_base: Option<String>,
    internal_secret: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiCredentials {
    client_id: String,
    token: String,
}

pub struct Config {
    pub discord_token: String,
    pub discord_client_id: u64,
    pub api_base: Option<String>,
    pub internal_secret: Option<String>,
}

pub async fn load() -> Result<Config, ConfigError> {
    let env: EnvConfig = envy::from_env()?;

    let api_base = env.tahti_api_base.clone().filter(|s| !s.is_empty());
    let internal_secret = env.internal_secret.clone().filter(|s| !s.is_empty());

    if let (Some(base), Some(secret)) = (api_base.as_deref(), internal_secret.as_deref()) {
        match fetch_from_api_with_retry(base, secret, &API_RETRY_DELAYS).await {
            Ok(mut config) => {
                config.api_base = api_base;
                config.internal_secret = internal_secret;
                return Ok(config);
            }
            Err(error) => {
                tracing::warn!(%error, "Tahti API Discord credentials unavailable, using env");
            }
        }
    }

    let mut config = from_env(env)?;
    config.api_base = api_base;
    config.internal_secret = internal_secret;
    Ok(config)
}

fn from_env(env: EnvConfig) -> Result<Config, ConfigError> {
    let discord_client_id = env
        .discord_client_id
        .filter(|s| !s.is_empty())
        .ok_or(ConfigError::Missing("DISCORD_CLIENT_ID"))?
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidClientId)?;

    Ok(Config {
        discord_token: env
            .discord_token
            .filter(|s| !s.is_empty())
            .ok_or(ConfigError::Missing("DISCORD_TOKEN"))?,
        discord_client_id,
        api_base: None,
        internal_secret: None,
    })
}

#[derive(Debug, thiserror::Error)]
enum FetchError {
    /// Worth another try: the API was unreachable, timed out or answered 5xx.
    #[error("{0}")]
    Transient(String),
    /// Retrying will not help: credentials are not configured (404), the
    /// secret is wrong (401) or the response is malformed.
    #[error("{0}")]
    Permanent(String),
}

fn is_transient_status(status: reqwest::StatusCode) -> bool {
    status.is_server_error()
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status == reqwest::StatusCode::REQUEST_TIMEOUT
}

async fn fetch_from_api_with_retry(
    base: &str,
    secret: &str,
    delays: &[Duration],
) -> Result<Config, FetchError> {
    let mut attempt = 0;
    loop {
        match fetch_from_api(base, secret).await {
            Ok(config) => {
                if attempt > 0 {
                    tracing::info!(attempt, "Tahti API Discord credentials loaded after retry");
                }
                return Ok(config);
            }
            Err(FetchError::Transient(error)) if attempt < delays.len() => {
                let delay = delays[attempt];
                attempt += 1;
                tracing::warn!(%error, attempt, retry_in_secs = delay.as_secs(), "Tahti API Discord credentials request failed, retrying");
                tokio::time::sleep(delay).await;
            }
            Err(error) => return Err(error),
        }
    }
}

async fn fetch_from_api(base: &str, secret: &str) -> Result<Config, FetchError> {
    let url = format!(
        "{}/api/v1/internal/discord-bot/credentials",
        base.trim_end_matches('/')
    );
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(secret)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|error| FetchError::Transient(error.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let message = format!("credentials request returned {status}");
        return Err(if is_transient_status(status) {
            FetchError::Transient(message)
        } else {
            FetchError::Permanent(message)
        });
    }

    let credentials = response
        .json::<ApiCredentials>()
        .await
        .map_err(|error| FetchError::Permanent(error.to_string()))?;

    let discord_client_id = credentials
        .client_id
        .parse::<u64>()
        .map_err(|_| FetchError::Permanent(ConfigError::InvalidClientId.to_string()))?;

    Ok(Config {
        discord_token: credentials.token,
        discord_client_id,
        api_base: None,
        internal_secret: None,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    const OK_BODY: &str = r#"{"clientId":"123456789","token":"api-token"}"#;

    /// Serves `responses` in order (status, body), one per connection, and
    /// counts the requests it saw.
    async fn serve(responses: Vec<(u16, &'static str)>) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&hits);
        tokio::spawn(async move {
            for (status, body) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 2048];
                let _ = socket.read(&mut buf).await;
                counter.fetch_add(1, Ordering::SeqCst);
                let reply = format!(
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(reply.as_bytes()).await.unwrap();
            }
        });
        (base, hits)
    }

    const NO_WAIT: [Duration; 3] = [Duration::ZERO; 3];

    #[tokio::test]
    async fn retries_until_the_api_answers() {
        let (base, hits) = serve(vec![(503, "{}"), (502, "{}"), (200, OK_BODY)]).await;
        let config = fetch_from_api_with_retry(&base, "secret", &NO_WAIT)
            .await
            .unwrap();
        assert_eq!(config.discord_token, "api-token");
        assert_eq!(config.discord_client_id, 123456789);
        assert_eq!(hits.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn does_not_retry_when_credentials_are_not_configured() {
        let (base, hits) = serve(vec![(404, "{}"), (200, OK_BODY)]).await;
        let result = fetch_from_api_with_retry(&base, "secret", &NO_WAIT).await;
        assert!(matches!(result, Err(FetchError::Permanent(_))));
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn gives_up_after_the_last_delay() {
        let (base, hits) = serve(vec![(503, "{}"); 4]).await;
        let result = fetch_from_api_with_retry(&base, "secret", &NO_WAIT).await;
        assert!(matches!(result, Err(FetchError::Transient(_))));
        assert_eq!(hits.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn treats_an_unreachable_api_as_transient() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let result = fetch_from_api_with_retry(&base, "secret", &[]).await;
        assert!(matches!(result, Err(FetchError::Transient(_))));
    }
}
