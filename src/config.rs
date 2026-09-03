use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub discord_client_id: u64,
}

pub fn load() -> Result<Config, envy::Error> {
    envy::from_env()
}
