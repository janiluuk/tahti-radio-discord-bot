use serde::Deserialize;

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
}

pub async fn load() -> Result<Config, ConfigError> {
    let env: EnvConfig = envy::from_env()?;

    if let (Some(base), Some(secret)) = (
        env.tahti_api_base.as_deref().filter(|s| !s.is_empty()),
        env.internal_secret.as_deref().filter(|s| !s.is_empty()),
    ) {
        match fetch_from_api(base, secret).await {
            Ok(config) => return Ok(config),
            Err(error) => {
                tracing::warn!(%error, "Tahti API Discord credentials unavailable, using env");
            }
        }
    }

    from_env(env)
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
    })
}

async fn fetch_from_api(base: &str, secret: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "{}/api/v1/internal/discord-bot/credentials",
        base.trim_end_matches('/')
    );
    let credentials = reqwest::Client::new()
        .get(url)
        .bearer_auth(secret)
        .send()
        .await?
        .error_for_status()?
        .json::<ApiCredentials>()
        .await?;

    let discord_client_id = credentials
        .client_id
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidClientId)?;

    Ok(Config {
        discord_token: credentials.token,
        discord_client_id,
    })
}
