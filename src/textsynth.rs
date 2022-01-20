use anyhow::Context;
use once_cell::sync::{Lazy, OnceCell};
use textsynth::core::TextSynth;
use textsynth::engine::Engine;
use crate::config;

static TEXT_SYNTH: OnceCell<TextSynth> = OnceCell::new();
static ENGINE: Lazy<Engine> = Lazy::new(|| get().engine(config::get().engine_definition.clone()));

pub fn initialize(api_key: String) -> anyhow::Result<&'static TextSynth> {
    TEXT_SYNTH.get_or_try_init(|| TextSynth::try_new(api_key))
        .context("failed to initialize the textsynth client")
}

pub fn get() -> &'static TextSynth {
    TEXT_SYNTH.get().expect("textsynth not initialized")
}

pub fn engine() -> &'static Engine<'static> {
    &ENGINE
}
