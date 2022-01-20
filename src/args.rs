use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Context;
use clap::Parser;
use tap::Pipe;
use textsynth::prelude::{NonEmptyString, Stop, TopK, TopP};

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
    }
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

pub fn parse() -> SynthText {
    SynthText::parse()
}
