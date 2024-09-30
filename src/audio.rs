use std::sync::Arc;

use anyhow::{anyhow, Error};
use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::SoundData,
};

pub struct AudioContext {
    audio_manager: AudioManager,
}

pub trait Audio {
    fn to_sound_data(&self) -> impl SoundData;
}

impl Audio for Arc<crate::asset::Audio> {
    fn to_sound_data(&self) -> impl SoundData {
        self.0.clone()
    }
}

impl AudioContext {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            audio_manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?,
        })
    }

    pub fn play(&mut self, source: impl Audio) -> Result<(), Error> {
        self.audio_manager
            .play(source.to_sound_data())
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }
}
