use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Service {
    pub port: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Tracer {
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub project: Project,
    pub service: Service,
    pub tracer: Tracer,
}

impl Config {
    pub fn parse(
        path: &std::path::PathBuf,
    ) -> std::result::Result<Config, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;

        Ok(config)
    }
}
