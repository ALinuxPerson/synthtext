mod text_completion;

use std::path::{Path, PathBuf};
use anyhow::Context;
use crate::{config, NonEmptyStringFromStrAdapter, SynthTextTextCompletionMethod, TopKFromStrAdapter, TopPFromStrAdapter};
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

fn existing(path: &Path) -> String {
    if path.exists() {
        "(existing)".green().italic().to_string()
    } else {
        "(non-existing)".red().italic().to_string()
    }
}

pub fn find_config_path(config_path_override: Option<PathBuf>) {
    let default_config_path = config::paths::location();

    match config_path_override {
        Some(config_path_override) => {
            alp::info!("the config path would be located at {} {}", default_config_path.display().bold(), existing(default_config_path));
            alp::info!("...but it was overridden to {} {}", config_path_override.display().bold(), existing(&config_path_override))
        },
        None => {
            alp::info!("the config path is located at {} {}", default_config_path.display().bold(), existing(default_config_path))
        }
    }
}
