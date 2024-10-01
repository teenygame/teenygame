use anyhow::{anyhow, Error};
use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::static_sound::StaticSoundData,
};

pub struct AudioContext {
    audio_manager: AudioManager,
}

impl AudioContext {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            audio_manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?,
        })
    }

    pub fn play(&mut self, source: StaticSoundData) -> Result<(), Error> {
        self.audio_manager
            .play(source)
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }
}
