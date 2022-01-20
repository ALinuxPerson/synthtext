mod text_completion;

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
