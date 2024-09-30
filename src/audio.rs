use std::sync::Arc;

use crate::asset::Audio;
use anyhow::{anyhow, Error};
use kira::manager::{AudioManager, AudioManagerSettings, DefaultBackend};

pub struct AudioContext {
    audio_manager: AudioManager,
}

impl AudioContext {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            audio_manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?,
        })
    }

    pub fn play(&mut self, source: &Arc<Audio>) -> Result<(), Error> {
        self.audio_manager
            .play(source.0.clone())
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }
}
