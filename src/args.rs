use anyhow::Context;
use clap::Parser;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::str::FromStr;
use tap::Pipe;
use textsynth::prelude::{
    CustomEngineDefinition, EngineDefinition, NonEmptyString, Stop, TopK, TopP,
};

/// A program which wraps the TextSynth API.
#[derive(Debug, Parser)]
pub struct SynthText {
    /// Override the local configuration file with the specified configuration file.
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
        NonEmptyString::new(s.into())
            .context("given string was empty")
            .map(Self)
    }
}

#[derive(Debug)]
pub struct TopKFromStrAdapter(pub TopK);

impl FromStr for TopKFromStrAdapter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u16>()
            .with_context(|| format!("the given string {} wasn't a valid number", s.bold()))?
            .pipe(TopK::new)
            .with_context(|| format!("the number {} wasn't in the required bound of 0..=1000", s.bold()))
            .map(Self)
    }
}

#[derive(Debug)]
pub struct TopPFromStrAdapter(pub TopP);

impl FromStr for TopPFromStrAdapter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>()
            .with_context(|| format!("the given string {} wasn't a valid float", s.bold()))?
            .pipe(TopP::new)
            .with_context(|| format!("the number {} wasn't in the required bound of 0.0..=1.0", s.bold()))
            .map(Self)
    }
}

#[derive(Debug)]
pub struct EngineDefinitionFromStrAdapter(pub EngineDefinition);

impl FromStr for EngineDefinitionFromStrAdapter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, max_tokens) = s
            .split_once(',')
            .map(|(id, max_tokens)| (id, Some(max_tokens)))
            .unwrap_or((s, None));
        let engine_definition = match (id, max_tokens) {
            ("gpt6jb", None) => EngineDefinition::GptJ6B,
            ("boris6b", None) => EngineDefinition::Boris6B,
            ("fairseqgpt13b", None) => EngineDefinition::FairseqGpt13B,
            (id, Some(max_tokens)) => {
                let max_tokens = max_tokens
                    .parse::<usize>()
                    .context("max tokens must be a valid number")?;
                EngineDefinition::Custom(CustomEngineDefinition::new(id.to_string(), max_tokens))
            }
            (_id, None) => anyhow::bail!(
                "expected delimiter {} to separate id and max tokens",
                ','.bold()
            ),
        };

        Ok(Self(engine_definition))
    }
}

#[derive(Debug, Parser)]
pub enum SynthTextAction {
    /// This action returns the logarithm of the probability that a continuation is generated
    /// after a context. It can be used to answer questions when only a few answers
    /// (such as yes/no) are possible. It can also be used to benchmark the models.
    #[clap(visible_aliases = &["lp", "l"])]
    LogProbabilities {
        /// If empty string, the context is set to the End-Of-Text token.
        context: String,

        /// Must be a non empty string.
        continuation: NonEmptyStringFromStrAdapter,
    },

    /// Completes and synthesizes text.
    #[clap(visible_aliases = &["tc", "t"])]
    TextCompletion {
        /// The input text to complete.
        prompt: String,

        /// Maximum number of tokens to generate. A token represents typically 4 or 5 characters
        /// for latin scripts.
        ///
        /// If the prompt length is larger than the model's maximum context length, the
        /// beginning of the prompt is discarded.
        #[clap(short, long)]
        max_tokens: Option<usize>,

        /// Sampling temperature. A higher temperature means the model will select less common
        /// tokens leading to a larger diversity but potentially less relevant output. It is
        /// usually better to tune top_p or top_k.
        #[clap(short, long)]
        temperature: Option<f64>,

        /// Select the next output token among the top_k most likely ones. A higher top_k gives
        /// more diversity but a potentially less relevant output.
        #[clap(short = 'k', long)]
        top_k: Option<TopKFromStrAdapter>,

        /// Select the next output token among the most probable ones so that their cumulative
        /// probability is larger than top_p. A higher top_p gives more diversity but a
        /// potentially less relevant output.
        #[clap(short = 'p', long)]
        top_p: Option<TopPFromStrAdapter>,

        /// How to run this text completion.
        #[clap(subcommand)]
        method: SynthTextTextCompletionMethod,
    },

    /// Generate or find the current configuration.
    #[clap(subcommand)]
    Config(SynthTextConfig),
}

#[derive(Debug, Parser)]
pub enum SynthTextTextCompletionMethod {
    /// Run this text completion now.
    #[clap(visible_alias = "n")]
    Now {
        /// Stop the generation when the string(s) are encountered. The generated text does not
        /// contain the string. The length of the array is at most 5.
        #[clap(short, long)]
        until: Vec<String>,
    },

    /// The output is streamed so that it is possible to display the result before the complete
    /// output is generated.
    #[clap(visible_alias = "s")]
    Stream,
}

#[derive(Debug, Parser)]
#[clap(visible_alias = "c")]
pub enum SynthTextConfig {
    /// Find the path of the configuration file, regardless of whether it exists or not.
    #[clap(visible_aliases = &["fp", "f"])]
    FindPath,

    /// Generate and write a configuration file.
    ///
    /// If no path was provided, it will be set to the default (or overridden by -c/--config) path
    /// (run `synthtext config find-path` to get it).
    ///
    /// As a precaution, if the file exists at the specified path, it will not continue. If this
    /// behavior is undesirable, pass the -c/--create argument.
    #[clap(visible_alias = "g")]
    Generate {
        /// The path of the configuration file.
        path: Option<PathBuf>,

        /// The API key used to authenticate into the API.
        #[clap(short, long)]
        api_key: String,

        /// The model or engine definition to use.
        #[clap(short, long)]
        engine_definition: Option<EngineDefinitionFromStrAdapter>,

        /// Do not write the configuration to a file. Instead, print it to stdout.
        #[clap(short, long)]
        dump: bool,

        /// Force creation of the configuration file to the specified location even if it exists.
        #[clap(short, long)]
        create: bool,
    },
}

pub fn parse() -> SynthText {
    SynthText::parse()
}
