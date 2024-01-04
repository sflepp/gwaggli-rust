use crate::audio::riff_wave::RiffWave;

pub mod fake;
pub mod whisper;

pub trait Transcribe {
    fn transcribe(&self, data: &RiffWave) -> String;
}