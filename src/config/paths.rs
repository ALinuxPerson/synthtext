use std::path::{Path, PathBuf};
use anyhow::Context;
use directories::ProjectDirs;
use once_cell::sync::{Lazy, OnceCell};

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "ALinuxPerson";
const APPLICATION: &str = "synthtext";
static PROJECT_DIRS: OnceCell<ProjectDirs> = OnceCell::new();
static DIRECTORY: Lazy<&Path> = Lazy::new(|| project_dirs().config_dir());
static LOCATION: Lazy<PathBuf> = Lazy::new(|| directory().join("config.json"));

pub fn initialize() -> anyhow::Result<()> {
    if PROJECT_DIRS.get().is_none() {
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .context("failed to initialize project directories")?;
        let _ = PROJECT_DIRS.set(project_dirs);
    }

    Ok(())
}

fn project_dirs() -> &'static ProjectDirs {
    PROJECT_DIRS.get().expect("project directories not initialized")
}

pub fn directory() -> &'static Path {
    &DIRECTORY
}

pub fn location() -> &'static Path {
    &LOCATION
}
