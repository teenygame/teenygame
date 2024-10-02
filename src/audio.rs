//! Audio support.

use std::time::Duration;

pub use kira::sound::FromFileError;
use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle},
        EndPosition, PlaybackPosition,
    },
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

/// Handle for controlling playback of a currently playing song.
///
/// Will stop playback when dropped.
pub struct PlaybackHandle(StaticSoundHandle);

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        self.stop(Tween::default());
    }
}

/// Easing motion of a tween.
#[derive(Clone, Copy)]
pub enum Easing {
    /// $f(x) = x$
    Linear,

    /// $f(x) = x^k$
    InPowi(i32),

    /// $f(x) = 1 - x^{1 - k}$
    OutPowi(i32),

    /// $f(x) = \begin{cases} \frac{(2x)^k}{2} & \text{if }x < 0.5\\\\ \frac{1 - (2 - 2x)^{k} + 1}{2} & \text{otherwise}\end{cases}$
    InOutPowi(i32),

    /// $f(x) = x^k$ (64-bit float precision)
    InPowf(f64),

    /// $f(x) = 1 - x^{1 - k}$ (64-bit float precision)
    OutPowf(f64),

    /// $f(x) = \begin{cases} \frac{(2x)^k}{2} & \text{if }x < 0.5\\\\ \frac{1 - (2 - 2x)^{k} + 1}{2} & \text{otherwise}\end{cases}$ (64-bit float precision)
    InOutPowf(f64),
}

impl From<Easing> for kira::tween::Easing {
    fn from(value: Easing) -> Self {
        match value {
            Easing::Linear => Self::Linear,
            Easing::InPowi(k) => Self::InPowi(k),
            Easing::OutPowi(k) => Self::OutPowi(k),
            Easing::InOutPowi(k) => Self::InOutPowi(k),
            Easing::InPowf(k) => Self::InPowf(k),
            Easing::OutPowf(k) => Self::OutPowf(k),
            Easing::InOutPowf(k) => Self::InOutPowf(k),
        }
    }
}

/// Describes a transition between values.
#[derive(Clone, Copy)]
pub struct Tween {
    /// Duration of the tween.
    pub duration: Duration,

    /// Easing function of the tween.
    pub easing: Easing,
}

impl From<Tween> for kira::tween::Tween {
    fn from(value: Tween) -> Self {
        Self {
            duration: value.duration,
            easing: value.easing.into(),
            ..Default::default()
        }
    }
}

impl Default for Tween {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(10),
            easing: Easing::Linear,
        }
    }
}

impl PlaybackHandle {
    pub fn stop(&mut self, tween: Tween) {
        self.0.stop(tween.into());
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
    pub fn play(&mut self, sound: &Sound) -> PlaybackHandle {
        let mut sound_data = sound.source.0.slice(Some(sound.playback_region.into()));

        if let Some(loop_region) = &sound.loop_region {
            sound_data = sound_data.loop_region(Some((*loop_region).into()));
        }

        PlaybackHandle(self.audio_manager.play(sound_data).unwrap())
    }
}
