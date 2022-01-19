use std::process;
use anyhow::Context;

mod config {
    use std::fs;
    use std::path::Path;
    use anyhow::Context;
    use once_cell::sync::OnceCell;
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

    static CONFIG: OnceCell<Config> = OnceCell::new();

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        pub api_key: String,
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
        Config::initialize()
    }

    pub fn initialize_with_location(location: &Path) -> anyhow::Result<&'static Config> {
        Config::initialize_with_location(location)
    }

    pub fn get() -> &'static Config {
        Config::get()
    }
}
mod args {
    use std::path::PathBuf;
    use std::str::FromStr;
    use anyhow::Context;
    use clap::Parser;
    use tap::Pipe;
    use textsynth::prelude::{NonEmptyString, Stop, TopK, TopP};

    #[derive(Debug, Parser)]
    pub struct SynthText {
        #[clap(short, long)]
        pub config: Option<PathBuf>,

        #[clap(subcommand)]
        pub action: SynthTextAction,
    }

    #[derive(Debug)]
    pub struct NonEmptyStringFromStrAdapter(pub NonEmptyString);

    impl FromStr for NonEmptyStringFromStrAdapter {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            NonEmptyString::new(s.into()).context("given string was empty").map(Self)
        }
    }

    #[derive(Debug)]
    pub struct TopKFromStrAdapter(pub TopK);

    impl FromStr for TopKFromStrAdapter {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u16>()
                .with_context(|| format!("the given string '{s}' wasn't a valid number"))?
                .pipe(TopK::new)
                .with_context(|| format!("the number {s} wasn't in the required bound of 0..=1000"))
                .map(Self)
        }
    }

    #[derive(Debug)]
    pub struct TopPFromStrAdapter(pub TopP);

    impl FromStr for TopPFromStrAdapter {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<f64>()
                .with_context(|| format!("the given string '{s}' wasn't a valid float"))?
                .pipe(TopP::new)
                .with_context(|| format!("the number {s} wasn't in the required bound of 0.0..=1.0"))
                .map(Self)
        }
    }

    #[derive(Debug, Parser)]
    pub enum SynthTextAction {
        LogProbabilities {
            context: String,
            continuation: NonEmptyStringFromStrAdapter,
        },
        TextCompletion {
            prompt: String,

            #[clap(short, long)]
            max_tokens: Option<usize>,

            #[clap(short, long)]
            temperature: Option<f64>,

            #[clap(short = 'k', long)]
            top_k: Option<TopKFromStrAdapter>,

            #[clap(short = 'p', long)]
            top_p: Option<TopKFromStrAdapter>,

            #[clap(subcommand)]
            method: SynthTextTextCompletionMethod,
        }
    }

    #[derive(Debug, Parser)]
    pub enum SynthTextTextCompletionMethod {
        Now,
        Stream {
            until: Vec<String>,
        }
    }

    pub fn parse() -> SynthText {
        SynthText::parse()
    }
}

fn main() {
    fn inner() -> anyhow::Result<()> {
        let args = args::parse();
        let config = match args.config {
            Some(config_path) => config::initialize_with_location(&config_path)
                .with_context(|| format!("failed to initialize the config with the specified location '{}'", config_path.display()))?,
            None => config::initialize()
                .context("failed to initialize the config with the default location")?,
        };

        Ok(())
    }

    let exit_code = match inner() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:#}", err);
            1
        }
    };

    process::exit(exit_code)
}
