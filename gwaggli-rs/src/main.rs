use std::fs::File;
use std::io::Read;
use transcription::whisper;
use whisper::{WhisperConfig, WhisperModel, WhisperTranscriber};
use crate::transcription::Transcribe;

mod audio;
mod transcription;

fn main() {
    let now = std::time::Instant::now();

    let file_path = "test_data/audio/riff_wave/OSR_us_000_0031_8k.wav";

    let mut file = File::open(file_path).expect("File not found");

    let mut audio_data = Vec::new();
    file.read_to_end(&mut audio_data).expect("Unable to read file");

    let riff_wave = audio::riff_wave::RiffWave::new(audio_data);

    let testee = WhisperTranscriber {
        config: WhisperConfig {
            model: WhisperModel::Tiny,
            model_dir: "./test_data/models/whisper".to_string(),
        }
    };

    let result = testee.transcribe(&riff_wave);

    println!("Elapsed: {:.2?} Result: {}",  now.elapsed(), result);
}
