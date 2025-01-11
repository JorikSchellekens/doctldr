use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default: DefaultConfig,
    pub api: ApiConfig,
    pub processing: ProcessingConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub model: String,
    pub max_tokens: usize,
    pub format: String,
    pub verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub provider: String,
    pub key_env: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub default_format: String,
    pub include_metadata: bool,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| ConfigError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find config directory",
            )))?
            .join("doctldr")
            .join("config.toml");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(config_path)?;
        Ok(toml::from_str(&contents)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: DefaultConfig {
                model: "gpt-4".to_string(),
                max_tokens: 2048,
                format: "md".to_string(),
                verbose: false,
            },
            api: ApiConfig {
                provider: "openai".to_string(),
                key_env: "OPENAI_API_KEY".to_string(),
            },
            processing: ProcessingConfig {
                include_patterns: vec![
                    "*.md".to_string(),
                    "*.rst".to_string(),
                    "*.txt".to_string(),
                    "*.html".to_string(),
                ],
                exclude_patterns: vec![
                    "node_modules".to_string(),
                    ".git".to_string(),
                ],
                max_depth: 5,
            },
            output: OutputConfig {
                default_format: "md".to_string(),
                include_metadata: true,
            },
        }
    }
} 