use std::process;

mod config {
    use std::fs;
    use anyhow::Context;
    use serde::{Deserialize, Serialize};
    use tap::Pipe;
    use textsynth::prelude::EngineDefinition;

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
        static LOCATION: OnceCell<PathBuf> = OnceCell::new();

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

        pub fn location() -> &'static Path {
            LOCATION.get_or_init(|| directory().join("config.json"))
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        pub api_key: String,
        pub engine_definition: EngineDefinition,
    }

    impl Config {
        pub fn load() -> anyhow::Result<Self> {
            let location = paths::location();
            location.pipe(fs::read_to_string)
                .with_context(|| format!("failed to read path '{}'", location.display()))?
                .pipe_ref(|contents| serde_json::from_str(contents))
                .with_context(|| format!("failed to parse contents of path '{}' to json", location.display()))
        }
    }

    pub fn load() -> anyhow::Result<Config> {
        paths::initialize()?;
        Config::load()
    }
}
mod args {
    use clap::Parser;

    #[derive(Debug, Parser)]
    pub struct SynthText {
        pub api_key: Option<String>,

        #[clap(subcommand)]
        pub action: SynthTextAction,
    }

    #[derive(Debug, Parser)]
    pub enum SynthTextAction {

    }
}

fn main() {
    fn inner() -> anyhow::Result<()> { Ok(()) }

    let exit_code = match inner() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:#}", err);
            1
        }
    };

    process::exit(exit_code)
}
