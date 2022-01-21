pub mod paths;

use anyhow::Context;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{fs, io};
use std::io::Write;
use std::path::Path;
use owo_colors::OwoColorize;
use tap::Pipe;
use textsynth::prelude::EngineDefinition;

static CONFIG: OnceCell<Config> = OnceCell::new();

const fn default_engine_definition() -> EngineDefinition {
    Config::DEFAULT_ENGINE_DEFINITION
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,

    #[serde(default = "default_engine_definition")]
    pub engine_definition: EngineDefinition,
}

impl Config {
    pub const DEFAULT_ENGINE_DEFINITION: EngineDefinition = EngineDefinition::GptJ6B;

    pub fn initialize() -> anyhow::Result<&'static Self> {
        CONFIG.get_or_try_init(Self::load)
    }

    pub fn initialize_with_location(location: &Path) -> anyhow::Result<&'static Self> {
        CONFIG.get_or_try_init(|| Self::load_with_location(location))
    }

    pub fn load() -> anyhow::Result<Self> {
        Self::load_with_location(paths::location())
    }

    pub fn load_with_location(location: &Path) -> anyhow::Result<Self> {
        let result = fs::read_to_string(location);

        if let Err(error) = &result {
            if let io::ErrorKind::NotFound = error.kind() {
                alp::tip!("generate the configuration file first");
                alp::tip!("synthtext config generate --api-key {}", "<API KEY>".italic());
            }
        }

        result
            .with_context(|| format!("failed to read path {}", location.display().bold()))?
            .pipe_ref(|contents| serde_json::from_str(contents))
            .with_context(|| {
                format!(
                    "failed to parse contents of path {} to json",
                    location.display().bold()
                )
            })
    }

    pub fn get() -> &'static Self {
        CONFIG.get().expect("config not initialized")
    }

    pub fn write(&self, mut writer: impl Write) -> anyhow::Result<()> {
        let contents = serde_json::to_string_pretty(self).context("failed to serialize config")?;
        let contents = contents.as_bytes();

        writer.write_all(contents).context("failed to write config")
    }
}

pub fn load() -> anyhow::Result<Config> {
    paths::initialize()?;
    Config::load()
}

pub fn load_with_location(location: &Path) -> anyhow::Result<Config> {
    paths::initialize()?;
    Config::load_with_location(location)
}

pub fn initialize() -> anyhow::Result<&'static Config> {
    paths::initialize()?;
    Config::initialize()
}

pub fn initialize_with_location(location: &Path) -> anyhow::Result<&'static Config> {
    paths::initialize()?;
    Config::initialize_with_location(location)
}

pub fn get() -> &'static Config {
    Config::get()
}
