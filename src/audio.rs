//! Audio support.

use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::static_sound::StaticSoundData,
};

/// Context for playing audio.
pub struct AudioContext {
    audio_manager: AudioManager,
}

impl AudioContext {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            audio_manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?,
        })
    }

    /// Start playing some audio.
    pub fn play(&mut self, source: StaticSoundData) {
        self.audio_manager.play(source).unwrap();
    }
}
