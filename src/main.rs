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
    }

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
        #[clap(visible_aliases = &["lp", "l"])]
        LogProbabilities {
            context: String,
            continuation: NonEmptyStringFromStrAdapter,
        },

        #[clap(visible_aliases = &["tc", "t"])]
        TextCompletion {
            prompt: String,

            #[clap(short, long)]
            max_tokens: Option<usize>,

            #[clap(short, long)]
            temperature: Option<f64>,

            #[clap(short = 'k', long)]
            top_k: Option<TopKFromStrAdapter>,

            #[clap(short = 'p', long)]
            top_p: Option<TopPFromStrAdapter>,

            #[clap(subcommand)]
            method: SynthTextTextCompletionMethod,
        }
    }

    #[derive(Debug, Parser)]
    pub enum SynthTextTextCompletionMethod {
        #[clap(visible_alias = "n")]
        Now {
            #[clap(short, long)]
            until: Vec<String>,
        },

        #[clap(visible_alias = "s")]
        Stream,
    }

    pub fn parse() -> SynthText {
        SynthText::parse()
    }
}
mod textsynth {
    use once_cell::sync::{Lazy, OnceCell};
    use textsynth::core::TextSynth;
    use textsynth::engine::Engine;
    use crate::config;

    static TEXT_SYNTH: OnceCell<TextSynth> = OnceCell::new();
    static ENGINE: Lazy<Engine> = Lazy::new(|| get().engine(config::get().engine_definition.clone()));

    pub fn initialize(api_key: String) -> &'static TextSynth {
        TEXT_SYNTH.get_or_init(|| TextSynth::new(api_key))
    }

    pub fn get() -> &'static TextSynth {
        TEXT_SYNTH.get().expect("textsynth not initialized")
    }

    pub fn engine() -> &'static Engine<'static> {
        &ENGINE
    }
}
mod app {
    mod text_completion {
        use std::ops::DerefMut;
        use std::pin::Pin;
        use std::{io, task};
        use std::io::Write;
        use std::task::Poll;
        use anyhow::Context;
        use futures::{Stream, StreamExt};
        use tap::{Pipe, Tap, TryConv};
        use textsynth::prelude::{MaxTokens, Stop, TextCompletionBuilder, TextCompletionStreamResult};
        use crate::{TopKFromStrAdapter, TopPFromStrAdapter};

        fn common(
            prompt: String,
            max_tokens: Option<usize>,
            temperature: Option<f64>,
            top_k: Option<TopKFromStrAdapter>,
            top_p: Option<TopPFromStrAdapter>,
        ) -> anyhow::Result<TextCompletionBuilder<'static, 'static>> {
            let engine = crate::textsynth::engine();
            let max_tokens: Option<MaxTokens> = match max_tokens {
                Some(max_tokens) => MaxTokens::new(max_tokens, &engine.definition)
                    .with_context(|| format!("the maximum number of tokens given, {max_tokens}, is not enough to fit in the engine definition (maximum supported for current engine definition is {})", engine.definition.max_tokens()))?
                    .pipe(Some),
                None => None,
            };
            let top_k = top_k.map(|top_k| top_k.0);
            let top_p = top_p.map(|top_p| top_p.0);
            engine.text_completion(prompt)
                .tap_mut(|this| this.max_tokens = max_tokens)
                .tap_mut(|this| this.temperature = temperature)
                .tap_mut(|this| this.top_k = top_k)
                .tap_mut(|this| this.top_p = top_p)
                .pipe(Ok)
        }

        pub async fn now(
            prompt: String,
            max_tokens: Option<usize>,
            temperature: Option<f64>,
            top_k: Option<TopKFromStrAdapter>,
            top_p: Option<TopPFromStrAdapter>,
            until: Vec<String>,
        ) -> anyhow::Result<()> {
            let until = if until.is_empty() {
                None
            } else {
                until.as_slice()
                    .try_conv::<Stop>()
                    .with_context(|| format!("passed overflowing 'until' argument; expected <= 5 items but got {}", until.len()))
                    .map(Some)?
            };
            let builder = common(prompt.clone(), max_tokens, temperature, top_k, top_p)?;
            let text_completion = match until {
                Some(until) => builder.now_until(until)
                    .await
                    .context("failed to connect to the textsynth api")?
                    .context("failed to generate a text completion now")?,
                None => builder.now()
                    .await
                    .context("failed to connect to the textsynth api")?
                    .context("failed to generate a text completion now")?,
            };
            print!("{}", prompt);

            println!("{}", text_completion.text());

            Ok(())
        }

        enum DynStream<T, U>
        where
            T: Stream<Item = TextCompletionStreamResult>,
            U: Stream<Item = TextCompletionStreamResult>,
        {
            Left(T),
            Right(U),
        }

        impl<T, U> Stream for DynStream<T, U>
            where
                T: Stream<Item = TextCompletionStreamResult> + Unpin,
                U: Stream<Item = TextCompletionStreamResult> + Unpin,
        {
            type Item = TextCompletionStreamResult;

            fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
                match self.deref_mut() {
                    Self::Left(left) => {
                        futures::pin_mut!(left);
                        left.poll_next(cx)
                    },
                    Self::Right(right) => {
                        futures::pin_mut!(right);
                        right.poll_next(cx)
                    },
                }
            }
        }

        pub async fn stream(
            prompt: String,
            max_tokens: Option<usize>,
            temperature: Option<f64>,
            top_k: Option<TopKFromStrAdapter>,
            top_p: Option<TopPFromStrAdapter>,
        ) -> anyhow::Result<()> {

            let builder = common(prompt.clone(), max_tokens, temperature, top_k, top_p)?;
            let mut stream = match until {
                Some(until) => DynStream::Left(builder.stream_until(until)
                    .await
                    .context("failed to connect to the textsynth api")?),
                None => DynStream::Right(builder.stream()
                    .await
                    .context("failed to connect to the textsynth api")?),
            };
            let mut stdout = io::stdout();
            print!("{}", prompt);
            stdout.flush().context("failed to flush stdout")?;

            while let Some(text_completion) = stream.next().await {
                let text_completion = text_completion
                    .context("failed to connect to textsynth api")?
                    .context("failed to parse output from textsynth api to json")?
                    .context("failed to get next text completion")?;

                print!("{}", text_completion.text());
                stdout.flush().context("failed to flush stdout")?;
            }

            println!();

            Ok(())
        }
    }

    use crate::{NonEmptyStringFromStrAdapter, SynthTextTextCompletionMethod, TopKFromStrAdapter, TopPFromStrAdapter};

    pub async fn log_probabilities(context: String, continuation: NonEmptyStringFromStrAdapter) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn text_completion(
        prompt: String,
        max_tokens: Option<usize>,
        temperature: Option<f64>,
        top_k: Option<TopKFromStrAdapter>,
        top_p: Option<TopPFromStrAdapter>,
        method: SynthTextTextCompletionMethod,
    ) -> anyhow::Result<()> {
        match method {
            SynthTextTextCompletionMethod::Now { until } => text_completion::now(
                prompt,
                max_tokens,
                temperature,
                top_k,
                top_p,
                until,
            ).await,
            SynthTextTextCompletionMethod::Stream => text_completion::stream(
                prompt,
                max_tokens,
                temperature,
                top_k,
                top_p,
            ).await,
        }
    }
}

use args::*;

#[tokio::main]
async fn main() {
    async fn inner() -> anyhow::Result<()> {
        let args = args::parse();
        let config = match args.config {
            Some(config_path) => config::initialize_with_location(&config_path)
                .with_context(|| format!("failed to initialize the config with the specified location '{}'", config_path.display()))?,
            None => config::initialize()
                .context("failed to initialize the config with the default location")?,
        };
        textsynth::initialize(config.api_key.clone());

        match args.action {
            SynthTextAction::LogProbabilities {
                context,
                continuation,
            } => app::log_probabilities(context, continuation).await,
            SynthTextAction::TextCompletion {
                prompt,
                max_tokens,
                temperature,
                top_k,
                top_p,
                method,
            } => app::text_completion(prompt, max_tokens, temperature, top_k, top_p, method).await,
        }
    }

    let exit_code = match inner().await {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:#}", err);
            1
        }
    };

    process::exit(exit_code)
}
