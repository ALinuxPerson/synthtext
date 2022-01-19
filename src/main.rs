mod config {
    mod paths {
        use std::path::{Path, PathBuf};
        use anyhow::Context;
        use directories::ProjectDirs;
        use once_cell::sync::OnceCell;

        const QUALIFIER: &str = "com";
        const ORGANIZATION: &str = "ALinuxPerson";
        const APPLICATION: &str = "synthtext";
        static PROJECT_DIRS: OnceCell<ProjectDirs> = OnceCell::new();
        static DIRECTORY: OnceCell<&Path> = OnceCell::new();
        static API_KEY: OnceCell<PathBuf> = OnceCell::new();

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
            DIRECTORY.get_or_init(|| project_dirs().config_dir())
        }

        pub fn api_key() -> &'static Path {
            API_KEY.get_or_init(|| directory().join("api_key"))
        }
    }
    mod api_key {
        use anyhow::Context;
        use once_cell::sync::OnceCell;
        use crate::config::paths;

        static API_KEY: OnceCell<String> = OnceCell::new();

        pub fn initialize() -> anyhow::Result<()> {
            if API_KEY.get().is_none() {
                let api_key = std::fs::read_to_string(paths::api_key())
                    .context("failed to read api key")?;
                let _ = API_KEY.set(api_key);
            }

            Ok(())
        }

        pub fn get() -> &'static str {
            API_KEY.get().expect("api key not initialized")
        }
    }
}

fn main() {
    println!("Hello, world!");
}
