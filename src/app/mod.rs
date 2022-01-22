mod text_completion;
pub mod config {
    use crate::config::Config;
    use crate::EngineDefinitionFromStrAdapter;
    use anyhow::Context;
    use owo_colors::OwoColorize;

    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::{env, fs, io};
    use tap::Tap;

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
                alp::info!(
                    "the config path {} be located at {} {}",
                    "would".italic(),
                    default_config_path.display().bold(),
                    existing(default_config_path)
                );
                alp::info!(
                    "...but it was overridden to {} {}",
                    config_path_override.display().bold(),
                    existing(&config_path_override)
                )
            }
            None => {
                alp::info!(
                    "the config path is located at {} {}",
                    default_config_path.display().bold(),
                    existing(default_config_path)
                )
            }
        }
    }

    enum FileOrStdout {
        File { handle: fs::File, path: PathBuf },
        Stdout(io::Stdout),
    }

    impl io::Write for FileOrStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self {
                FileOrStdout::File { handle, .. } => handle.write(buf),
                FileOrStdout::Stdout(stdout) => stdout.write(buf),
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self {
                FileOrStdout::File { handle, .. } => handle.flush(),
                FileOrStdout::Stdout(stdout) => stdout.flush(),
            }
        }
    }

    pub fn generate(
        config_path_override: Option<PathBuf>,
        path: Option<PathBuf>,
        api_key: String,
        engine_definition: Option<EngineDefinitionFromStrAdapter>,
        dump: bool,
        create: bool,
    ) -> anyhow::Result<()> {
        let mut writer = if dump {
            FileOrStdout::Stdout(io::stdout())
        } else {
            let path = path
                .or(config_path_override)
                .unwrap_or_else(|| crate::config::paths::location().to_path_buf());

            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    alp::warn!(
                        "parent directory {} does not exist, creating it",
                        parent.display().bold()
                    );
                    fs::create_dir_all(parent).with_context(|| {
                        format!(
                            "failed to create parent directory {}",
                            parent.display().bold()
                        )
                    })?;
                }
            }

            let mut builder = fs::OpenOptions::new().tap_mut(|this| {
                this.write(true);
            });

            if create {
                builder.create(true).truncate(true);
            } else {
                builder.create_new(true);
            };

            let result = builder.open(&path);

            if let Err(error) = &result {
                if let io::ErrorKind::AlreadyExists = error.kind() {
                    let c_create = "-c/--create".bold();
                    alp::tip!("as a precaution, writing a config file fails if it already exists. if this behavior is undesirable, pass the {c_create} argument in your command.");
                    let command = env::args()
                        .map(|argument| {
                            if argument == api_key {
                                "<API KEY REDACTED>".to_string()
                            } else {
                                argument
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    alp::tip!("short variant: {}", format_args!("{} -c", command).italic());
                    alp::tip!(
                        "long variant: {}",
                        format_args!("{} --create", command).italic()
                    );
                }
            }

            let handle =
                result.with_context(|| format!("failed to open path {}", path.display().bold()))?;

            FileOrStdout::File { handle, path }
        };
        let engine_definition = engine_definition.map(|engine_definition| engine_definition.0);
        let config = Config {
            api_key,
            engine_definition: engine_definition.unwrap_or(Config::DEFAULT_ENGINE_DEFINITION),
        };

        config.write(&mut writer).with_context(|| match &writer {
            FileOrStdout::File { path, .. } => {
                format!("failed to write to file {}", path.display().bold())
            }
            FileOrStdout::Stdout(_) => format!("failed to write to {}", "stdout".bold()),
        })?;

        match writer {
            FileOrStdout::File { path, .. } => {
                alp::info!("generated config file at {}", path.display().bold())
            }
            FileOrStdout::Stdout(_) => {
                // write an extra new line if stdout to prevent unterminated lines
                writer.write_all(&[b'\n']).with_context(|| {
                    format!("failed to write new line into {}", "stdout".bold())
                })?;
            }
        }

        Ok(())
    }
}

use crate::{InfallibleFromStr, NonEmptyStringFromStrAdapter, Prompt, SynthTextTextCompletionMethod, TopKFromStrAdapter, TopPFromStrAdapter};
use anyhow::Context;
use owo_colors::OwoColorize;

pub async fn log_probabilities(
    context: String,
    NonEmptyStringFromStrAdapter(continuation): NonEmptyStringFromStrAdapter,
) -> anyhow::Result<()> {
    alp::info!("the provided context was: '{context}'");
    alp::info!("the predicted continuation was: '{}'", continuation.inner());

    let log_probabilities = crate::textsynth::engine()
        .log_probabilities(context, continuation)
        .await
        .context("failed to connect to the textsynth api")?
        .context("failed to get log probabilities")?;

    alp::info!(
        "log probability: {}",
        log_probabilities.log_probability().bold()
    );
    alp::info!("is greedy: {}", log_probabilities.is_greedy().bold());
    alp::info!("total tokens: {}", log_probabilities.total_tokens().bold());

    Ok(())
}

pub async fn text_completion(
    prompt: InfallibleFromStr<Prompt>,
    max_tokens: Option<usize>,
    temperature: Option<f64>,
    top_k: Option<TopKFromStrAdapter>,
    top_p: Option<TopPFromStrAdapter>,
    method: SynthTextTextCompletionMethod,
) -> anyhow::Result<()> {
    match method {
        SynthTextTextCompletionMethod::Now { until } => {
            text_completion::now(prompt, max_tokens, temperature, top_k, top_p, until).await
        }
        SynthTextTextCompletionMethod::Stream => {
            text_completion::stream(prompt, max_tokens, temperature, top_k, top_p).await
        }
    }
}
