//! Audio support.

pub use kira::sound::FromFileError;
use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::{static_sound::StaticSoundData, EndPosition, PlaybackPosition, Region},
};

/// A source of sound data.
pub struct Source(StaticSoundData);

/// A looping region of sound, in samples.
pub struct LoopRegion {
    /// Start position of the loop region, in samples.
    pub start: u64,

    /// Length of the loop region, in samples.
    pub length: u64,
}

impl Source {
    /// Reads sound data from a buffer.
    ///
    /// This will also perform decoding, depending on what codecs are available. This uses [Symphonia](https://github.com/pdeljanov/Symphonia) internally.
    pub fn from_raw(buf: &[u8]) -> Result<Self, FromFileError> {
        Ok(Self(StaticSoundData::from_cursor(std::io::Cursor::new(
            buf.to_vec(),
        ))?))
    }
}

/// A sound.
///
/// This plays the underlying source with various parameters.
pub struct Sound<'a> {
    /// The source to play.
    pub source: &'a Source,

    /// The region to loop infinitely, if any.
    pub loop_region: Option<LoopRegion>,

    /// The position to start playback at, in samples.
    pub start_position: u64,
}

impl<'a> Sound<'a> {
    /// Creates a new Sound with no extra parameters.
    pub fn new(source: &'a Source) -> Self {
        Self {
            source,
            loop_region: None,
            start_position: 0,
        }
    }
}

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

    /// Plays a sound.
    pub fn play(&mut self, sound: &Sound) {
        let mut sound_data = sound
            .source
            .0
            .start_position(PlaybackPosition::Samples(sound.start_position as usize));

        if let Some(loop_region) = &sound.loop_region {
            sound_data = sound_data.loop_region(Region {
                start: PlaybackPosition::Samples(loop_region.start as usize),
                end: EndPosition::Custom(PlaybackPosition::Samples(
                    (loop_region.start + loop_region.length) as usize,
                )),
            })
        }

        self.audio_manager.play(sound_data).unwrap();
    }
}
