use rodio::{OutputStream, OutputStreamHandle};
pub use rodio::{PlayError, Source, StreamError};

pub struct AudioContext {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl AudioContext {
    pub(crate) fn new() -> Result<Self, StreamError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            stream_handle,
        })
    }

    pub fn play<S>(&self, source: S) -> Result<(), PlayError>
    where
        S: Source<Item = f32> + Send + 'static,
    {
        self.stream_handle.play_raw(source)
    }
}
