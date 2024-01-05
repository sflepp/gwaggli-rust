use crate::transcription::Transcribe;
use std::fs::File;
use std::io::Read;
use transcription::whisper;
use whisper::{WhisperConfig, WhisperModel, WhisperTranscriber};

mod audio;
mod transcription;
mod environment;

fn main() {
    let file_path = "test_data/audio/riff_wave/pcm_s16le_16k_mono.wav";

    let mut file = File::open(file_path).expect("File not found");

    let mut audio_data = Vec::new();
    file.read_to_end(&mut audio_data)
        .expect("Unable to read file");

    let riff_wave = audio::riff_wave::RiffWave::new(audio_data).unwrap();

    let mut testee = WhisperTranscriber::new(WhisperConfig {
        model: WhisperModel::Tiny,
    });

    testee.load_context().unwrap();

    let now = std::time::Instant::now();

    let result = testee.transcribe(&riff_wave).unwrap();

    println!("Elapsed: {:.2?} Result: {}", now.elapsed(), result);

    let now = std::time::Instant::now();

    let result = testee.transcribe(&riff_wave).unwrap();

    println!("Elapsed: {:.2?} Result: {}", now.elapsed(), result);
}
