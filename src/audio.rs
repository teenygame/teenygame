//! Audio support.

use std::time::Duration;

pub use kira::sound::FromFileError;
use kira::{
    manager::{AudioManager, AudioManagerSettings, DefaultBackend},
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle},
        EndPosition, PlaybackPosition, PlaybackRate,
    },
    Volume,
};

/// A source of sound data.
pub struct Source(StaticSoundData);

impl Source {
    /// Gets the sample rate.
    pub fn sample_rate(&self) -> usize {
        self.0.sample_rate as usize
    }

    /// Gets the number of frames.
    pub fn num_frames(&self) -> usize {
        self.0.num_frames()
    }

    /// Gets a duration in samples.
    pub fn to_samples(&self, dur: Duration) -> usize {
        self.sample_rate() * dur.as_secs() as usize
    }

    /// Gets a sample count as a duration.
    pub fn to_duration(&self, n: usize) -> Duration {
        Duration::from_secs_f64(n as f64 / self.sample_rate() as f64)
    }

    /// Gets the duration of the source.
    pub fn duration(&self) -> Duration {
        self.to_duration(self.num_frames())
    }
}

/// A region of sound, in samples.
#[derive(Clone, Copy)]
pub struct Region {
    /// Start position of the region, in samples.
    pub start: usize,

    /// Length of the region, in samples.
    pub length: usize,
}

impl Region {
    fn into_impl(self) -> kira::sound::Region {
        kira::sound::Region {
            start: PlaybackPosition::Samples(self.start as usize),
            end: EndPosition::Custom(PlaybackPosition::Samples(
                (self.start + self.length) as usize,
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

/// Handle for controlling playback of a currently playing sound.
///
/// Will stop playback when dropped, unless detached.
pub struct PlaybackHandle(Option<StaticSoundHandle>);

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        if self.0.is_none() {
            return;
        }
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

impl Easing {
    fn into_impl(self) -> kira::tween::Easing {
        match self {
            Self::Linear => kira::tween::Easing::Linear,
            Self::InPowi(k) => kira::tween::Easing::InPowi(k),
            Self::OutPowi(k) => kira::tween::Easing::OutPowi(k),
            Self::InOutPowi(k) => kira::tween::Easing::InOutPowi(k),
            Self::InPowf(k) => kira::tween::Easing::InPowf(k),
            Self::OutPowf(k) => kira::tween::Easing::OutPowf(k),
            Self::InOutPowf(k) => kira::tween::Easing::InOutPowf(k),
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

impl Tween {
    fn into_impl(self) -> kira::tween::Tween {
        kira::tween::Tween {
            duration: self.duration,
            easing: self.easing.into_impl(),
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
    /// Stops playback.
    pub fn stop(&mut self, tween: Tween) {
        self.0.as_mut().unwrap().stop(tween.into_impl());
    }

    /// Set panning of the audio, where -1.0 is hard left and 1.0 is hard right.
    pub fn set_panning(&mut self, panning: f64, tween: Tween) {
        self.0
            .as_mut()
            .unwrap()
            .set_panning((panning - 0.5) * 2.0, tween.into_impl());
    }

    /// Set volume of the audio, where the volume is the multiplier of the amplitude.
    pub fn set_volume(&mut self, volume: f64, tween: Tween) {
        self.0
            .as_mut()
            .unwrap()
            .set_volume(Volume::Amplitude(volume), tween.into_impl());
    }

    /// Set speed of the audio, where the speed is the multiplier of the play speed.
    pub fn set_speed(&mut self, speed: f64, tween: Tween) {
        self.0
            .as_mut()
            .unwrap()
            .set_playback_rate(PlaybackRate::Factor(speed), tween.into_impl());
    }

    /// Detaches this playback such that it will continue playing.
    ///
    /// Note that this consumes the handle and it will be lost after detaching.
    pub fn detach(mut self) {
        self.0 = None;
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
        let mut sound_data = sound
            .source
            .0
            .slice(Some(sound.playback_region.into_impl()));

        if let Some(loop_region) = &sound.loop_region {
            sound_data = sound_data.loop_region(Some((*loop_region).into_impl()));
        }

        PlaybackHandle(Some(self.audio_manager.play(sound_data).unwrap()))
    }
}
