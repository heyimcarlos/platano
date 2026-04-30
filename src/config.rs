use serde;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub llm: LLMSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct LLMSettings {
    pub base_url: String,
    pub model: String,
    pub temperature: Option<f32>,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        match str.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!("{other} is not a supported environment")),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to get curren dir");
    let config_dir = base_path.join("config");

    let environment: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".to_string())
        .try_into()
        .expect("Failed to parse APP_ENV");

    let environment_filename = format!("{}.yaml", environment.as_str());

    config::Config::builder()
        .add_source(config::File::from(config_dir.join("base.yaml")))
        .add_source(config::File::from(config_dir.join(environment_filename)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize::<Settings>()
}
