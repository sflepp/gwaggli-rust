use crate::audio::riff_wave::RiffWave;
use crate::transcription::Transcribe;

pub struct FakeTranscriber {}

impl Transcribe for FakeTranscriber {
    fn transcribe(&self, data: &RiffWave) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!(
            "No real transcription, but returning some data. Length={}",
            data.data.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::transcription::Transcribe;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_transcribe() {
        let file_path = "test_data/audio/riff_wave/pcm_s16le_8k_mono.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data).unwrap();

        let testee = super::FakeTranscriber {};

        let result = testee.transcribe(&riff_wave).unwrap();

        assert_eq!(
            result,
            "No real transcription, but returning some data. Length=264014".to_string()
        );
    }
}
