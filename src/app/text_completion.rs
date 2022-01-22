use crate::{InfallibleFromStr, Prompt, TopKFromStrAdapter, TopPFromStrAdapter};
use anyhow::Context;
use futures::StreamExt;
use owo_colors::OwoColorize;
use std::io::Write;

use std::io;
use tap::{Pipe, Tap, TryConv};
use textsynth::prelude::{MaxTokens, Stop, TextCompletionBuilder};

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
    engine
        .text_completion(prompt)
        .tap_mut(|this| this.max_tokens = max_tokens)
        .tap_mut(|this| this.temperature = temperature)
        .tap_mut(|this| this.top_k = top_k)
        .tap_mut(|this| this.top_p = top_p)
        .pipe(Ok)
}

pub async fn now(
    InfallibleFromStr(prompt): InfallibleFromStr<Prompt>,
    max_tokens: Option<usize>,
    temperature: Option<f64>,
    top_k: Option<TopKFromStrAdapter>,
    top_p: Option<TopPFromStrAdapter>,
    until: Vec<String>,
) -> anyhow::Result<()> {
    let until = if until.is_empty() {
        None
    } else {
        until
            .as_slice()
            .try_conv::<Stop>()
            .with_context(|| {
                format!(
                    "passed overflowing {} argument; expected <= 5 items but got {}",
                    "until".bold(),
                    until.len()
                )
            })
            .map(Some)?
    };
    let prompt = prompt.into_string().context("failed to parse prompt into string")?;
    let builder = common(prompt.clone(), max_tokens, temperature, top_k, top_p)?;
    let text_completion = match until {
        Some(until) => builder
            .now_until(until)
            .await
            .context("failed to connect to the textsynth api")?
            .context("failed to generate a text completion now")?,
        None => builder
            .now()
            .await
            .context("failed to connect to the textsynth api")?
            .context("failed to generate a text completion now")?,
    };
    print!("{}", prompt);

    println!("{}", text_completion.text());

    if text_completion.truncated_prompt() {
        alp::warn!("prompt was truncated; the prompt was too large compared to the engine definition's maximum context length");
        alp::tip!(
            "try shortening your prompt to fit in the engine definition's maximum context length"
        );
    }

    if let Some(total_tokens) = text_completion.total_tokens() {
        alp::info!("total tokens used: {}", total_tokens.bold());
    }

    Ok(())
}

pub async fn stream(
    InfallibleFromStr(prompt): InfallibleFromStr<Prompt>,
    max_tokens: Option<usize>,
    temperature: Option<f64>,
    top_k: Option<TopKFromStrAdapter>,
    top_p: Option<TopPFromStrAdapter>,
) -> anyhow::Result<()> {
    let prompt = prompt.into_string().context("failed to parse prompt into string")?;
    let mut stream = common(prompt.clone(), max_tokens, temperature, top_k, top_p)?
        .stream()
        .await
        .context("failed to connect to the textsynth api")?;
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
