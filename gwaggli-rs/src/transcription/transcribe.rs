use crate::audio::riff_wave::RiffWave;
pub trait Transcribe {
    fn transcribe(&self, data: &RiffWave) -> String;
}