use crate::audio::riff_wave::RiffWave;
use std::error::Error;

pub mod fake;
pub mod whisper;

pub trait Transcribe {
    fn transcribe(&self, data: &RiffWave) -> Result<String, Box<dyn Error>>;
}
