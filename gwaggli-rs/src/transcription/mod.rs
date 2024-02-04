use crate::audio::riff_wave::RiffWave;
use std::error::Error;

pub mod fake;
pub mod real_time;
pub mod whisper;

pub trait Transcribe {
    fn transcribe(&self, data: &RiffWave) -> Result<String, Box<dyn Error>>;
}

pub trait TranscribeRaw {
    fn transcribe_raw(&self, data: Vec<f32>) -> Result<String, Box<dyn Error>>;
}
