mod text_completion;
pub mod config {
    use std::{fs, io};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use anyhow::Context;
    use owo_colors::OwoColorize;
    use crate::config::Config;
    use crate::EngineDefinitionFromStrAdapter;

    fn existing(path: &Path) -> String {
        if path.exists() {
            "(existing)".green().italic().to_string()
        } else {
            "(non-existing)".red().italic().to_string()
        }
    }

    pub fn find_path(config_path_override: Option<PathBuf>) {
        let default_config_path = crate::config::paths::location();

        match config_path_override {
            Some(config_path_override) => {
                alp::info!("the config path {} be located at {} {}", "would".italic(), default_config_path.display().bold(), existing(default_config_path));
                alp::info!("...but it was overridden to {} {}", config_path_override.display().bold(), existing(&config_path_override))
            },
            None => {
                alp::info!("the config path is located at {} {}", default_config_path.display().bold(), existing(default_config_path))
            }
        }
    }

    enum FileOrStdout {
        File {
            handle: fs::File,
            path: PathBuf,
        },
        Stdout(io::Stdout)
    }

    impl io::Write for FileOrStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self {
                FileOrStdout::File { handle, .. } => handle.write(buf),
                FileOrStdout::Stdout(stdout) => stdout.write(buf)
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self {
                FileOrStdout::File { handle, .. } => handle.flush(),
                FileOrStdout::Stdout(stdout) => stdout.flush()
            }
        }
    }

    pub fn generate(path: Option<PathBuf>, api_key: String, engine_definition: Option<EngineDefinitionFromStrAdapter>) -> anyhow::Result<()> {
        let mut writer = match path {
            Some(path) => {
                let handle = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .with_context(|| format!("failed to open path {}", path.display().bold()))?;

                FileOrStdout::File { handle, path }
            },
            None => {
                FileOrStdout::Stdout(io::stdout())
            }
        };
        let engine_definition = engine_definition.map(|engine_definition| engine_definition.0);
        let config = Config {
            api_key,
            engine_definition: engine_definition.unwrap_or(Config::DEFAULT_ENGINE_DEFINITION)
        };

        config.write(&mut writer)
            .with_context(|| {
                match &writer {
                    FileOrStdout::File { path, .. } => format!("failed to write to file {}", path.display().bold()),
                    FileOrStdout::Stdout(_) => format!("failed to write to {}", "stdout".bold()),
                }
            })?;

        match writer {
            FileOrStdout::File { path, .. } => {
                alp::info!("generated config file at {}", path.display().bold())
            },
            FileOrStdout::Stdout(_) => {
                // write an extra new line if stdout to prevent unterminated lines
                writer.write_all(&[b'\n'])
                    .with_context(|| format!("failed to write new line into {}", "stdout".bold()))?;
            }
        }

        Ok(())
    }
}

use std::path::{Path, PathBuf};
use anyhow::Context;
use crate::{NonEmptyStringFromStrAdapter, SynthTextTextCompletionMethod, TopKFromStrAdapter, TopPFromStrAdapter};
use owo_colors::OwoColorize;

pub async fn log_probabilities(context: String, NonEmptyStringFromStrAdapter(continuation): NonEmptyStringFromStrAdapter) -> anyhow::Result<()> {
    alp::info!("the provided context was: '{context}'");
    alp::info!("the predicted continuation was: '{}'", continuation.inner());

    let log_probabilities = crate::textsynth::engine()
        .log_probabilities(context, continuation)
        .await
        .context("failed to connect to the textsynth api")?
        .context("failed to get log probabilities")?;

    alp::info!("log probability: {}", log_probabilities.log_probability().bold());
    alp::info!("is greedy: {}", log_probabilities.is_greedy().bold());
    alp::info!("total tokens: {}", log_probabilities.total_tokens().bold());

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
