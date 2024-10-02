//! Audio support.

pub use kira::sound::FromFileError;
use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::{static_sound::StaticSoundData, EndPosition, PlaybackPosition},
};

/// A source of sound data.
pub struct Source(StaticSoundData);

/// A region of sound, in samples.
#[derive(Clone, Copy)]
pub struct Region {
    /// Start position of the loop region, in samples.
    pub start: usize,

    /// Length of the loop region, in samples.
    pub length: usize,
}

impl From<Region> for kira::sound::Region {
    fn from(value: Region) -> Self {
        Self {
            start: PlaybackPosition::Samples(value.start as usize),
            end: EndPosition::Custom(PlaybackPosition::Samples(
                (value.start + value.length) as usize,
            )),
        }
    }
}

impl Source {
    /// Load sound data from raw bytes.
    ///
    /// This will also perform decoding, depending on what codecs are available. This uses [Symphonia](https://github.com/pdeljanov/Symphonia) internally.
    pub fn load(buf: &[u8]) -> Result<Self, FromFileError> {
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
    pub loop_region: Option<Region>,

    /// The region to play.
    ///
    /// If [`Sound::loop_region`] is contained within the playback region, the playback will loop.
    pub playback_region: Region,
}

impl<'a> Sound<'a> {
    /// Creates a new Sound with no extra parameters.
    pub fn new(source: &'a Source) -> Self {
        Self {
            source,
            loop_region: None,
            playback_region: Region {
                start: 0,
                length: source.0.num_frames() as usize,
            },
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
        let mut sound_data = sound.source.0.slice(Some(sound.playback_region.into()));

        if let Some(loop_region) = &sound.loop_region {
            sound_data = sound_data.loop_region(Some((*loop_region).into()));
        }

        self.audio_manager.play(sound_data).unwrap();
    }
}
