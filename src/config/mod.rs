mod paths;

use std::fs;
use std::path::Path;
use anyhow::Context;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tap::Pipe;
use textsynth::prelude::EngineDefinition;

static CONFIG: OnceCell<Config> = OnceCell::new();

const fn default_engine_definition() -> EngineDefinition {
    EngineDefinition::GptJ6B
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,

    #[serde(default = "default_engine_definition")]
    pub engine_definition: EngineDefinition,
}

impl Config {
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
        location.pipe(fs::read_to_string)
            .with_context(|| format!("failed to read path '{}'", location.display()))?
            .pipe_ref(|contents| serde_json::from_str(contents))
            .with_context(|| format!("failed to parse contents of path '{}' to json", location.display()))
    }

    pub fn get() -> &'static Self {
        CONFIG.get().expect("config not initialized")
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
