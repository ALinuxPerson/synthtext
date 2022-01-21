mod app;
mod args;
mod config;
mod textsynth;

use anyhow::Context;
use args::*;
use std::process;
use tap::Pipe;

#[tokio::main]
async fn main() {
    async fn inner() -> anyhow::Result<()> {
        let args = args::parse();

        config::paths::initialize().context("failed to initialize config paths")?;

        if !matches!(args.action, SynthTextAction::Config(_)) {
            let config = match args.config {
                Some(ref config_path) => config::initialize_with_location(config_path)
                    .with_context(|| {
                        format!(
                            "failed to initialize the config with the specified location '{}'",
                            config_path.display()
                        )
                    })?,
                None => config::initialize()
                    .context("failed to initialize the config with the default location")?,
            };

            textsynth::initialize(config.api_key.clone())?;
        }

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
            SynthTextAction::Config(config) => match config {
                #[allow(clippy::unit_arg)]
                SynthTextConfig::FindPath => app::config::find_path(args.config).pipe(Ok),

                SynthTextConfig::Generate {
                    path,
                    api_key,
                    engine_definition,
                    dump,
                    create,
                } => app::config::generate(
                    args.config,
                    path,
                    api_key,
                    engine_definition,
                    dump,
                    create,
                ),
            },
        }
    }

    let exit_code = match inner().await {
        Ok(_) => 0,
        Err(err) => {
            alp::error!("{:#}", err);
            1
        }
    };

    process::exit(exit_code)
}
