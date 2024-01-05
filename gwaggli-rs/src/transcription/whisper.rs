extern crate curl;

use crate::audio::riff_wave::Channels::Mono;
use crate::audio::riff_wave::RiffWave;
use crate::transcription::Transcribe;
use curl::easy::Easy;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{PathBuf};
use dirs::{cache_dir};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperTranscriber {
    pub config: WhisperConfig,
    context: Option<WhisperContext>,
}

pub struct WhisperConfig {
    pub model: WhisperModel,
}

#[allow(dead_code)]
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl WhisperModel {
    pub fn get_model_name(&self) -> &str {
        match self {
            WhisperModel::Tiny => "ggml-tiny.en.bin",
            WhisperModel::Base => "ggml-base.bin",
            WhisperModel::Small => "ggml-small.bin",
            WhisperModel::Medium => "ggml-medium.bin",
            WhisperModel::Large => "ggml-large-v3.bin",
        }
    }
    pub fn get_model_url(&self) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
            self.get_model_name()
        )
    }

    pub fn download(&self) -> Result<PathBuf, Box<dyn Error>> {
        let cache_dir = cache_dir().expect("Unable to get cache dir");
        let model_dir = cache_dir.join("gwaggli-rs/models/whisper");
        let model_path = model_dir.join(self.get_model_name());

        if !fs::metadata(&model_dir).is_ok() {
            fs::create_dir_all(&model_dir)?;
        }

        if fs::metadata(&model_path).is_ok() {
            println!(
                "Model {} already exists, skipping download.",
                self.get_model_name()
            );
            return Ok(model_path);
        }

        println!(
            "Downloading model {} from {}",
            self.get_model_name(),
            self.get_model_url()
        );

        let mut dest = BufWriter::new(File::create(&model_path).unwrap());

        let url = self.get_model_url();

        let mut easy = Easy::new();

        easy.url(&url).unwrap();
        easy.follow_location(true).unwrap();

        easy.write_function(move |data| {
            dest.write_all(data).unwrap();
            Ok(data.len())
        })?;

        easy.perform().unwrap();

        let response_code = easy.response_code()?;

        if response_code >= 400 {
            return Err(
                format!("Error downloading model. Response code: {}", response_code).into(),
            );
        }

        Ok(model_path)
    }
}

impl WhisperTranscriber {
    pub fn new(config: WhisperConfig) -> Self {
        WhisperTranscriber {
            config,
            context: None,
        }
    }

    pub fn load_context(&mut self) -> Result<(), Box<dyn Error>> {
        let model = self
            .config
            .model
            .download()?;

        let ctx = WhisperContext::new_with_params(
            model.to_str().unwrap(),
            WhisperContextParameters::default(),
        )?;

        self.context = Some(ctx);

        Ok(())
    }
}

impl Transcribe for WhisperTranscriber {
    fn transcribe(&self, data: &RiffWave) -> Result<String, Box<dyn Error>> {
        if data.format.sample_rate != 16_000 {
            return Err(format!("Unsupported sample rate: {}", data.format.sample_rate,).into());
        }

        if data.format.num_channels != Mono {
            return Err(format!(
                "Unsupported number of channels: {}",
                data.format.num_channels,
            )
            .into());
        }


        let context = self.context.as_ref().expect("Context not loaded");

        let mut state = context.create_state()?;

        let mut params = FullParams::new(SamplingStrategy::default());
        params.set_n_threads(2);
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);

        state.full(params, &data.data_as_f32())?;

        let num_segments = state.full_n_segments()?;

        let mut result = String::from("");

        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("Failed to get segment text");

            result.push_str(&segment);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{WhisperConfig, WhisperModel, WhisperTranscriber};
    use crate::transcription::Transcribe;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_transcribe() {
        let file_path = "test_data/audio/riff_wave/pcm_s16le_16k_mono.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data).unwrap();

        let mut testee = WhisperTranscriber::new(WhisperConfig {
            model: WhisperModel::Tiny
        });

        testee.load_context().unwrap();

        let result = testee.transcribe(&riff_wave).unwrap();

        assert_eq!(result, " Plans are well underway for races to Mars and the Moon in 1992 by solar sales. The race to Mars is to commemorate Columbus's journey to the new world 500 years ago, and the launch of the Moon is to promote the use of solar sales in space exploration.".to_string());
    }

    #[test]
    fn test_get_model_url() {
        let models = [
            WhisperModel::Tiny,
            WhisperModel::Base,
            WhisperModel::Small,
            WhisperModel::Medium,
            WhisperModel::Large,
        ]
        .map(|m| m.get_model_url());

        assert_eq!(
            models[0],
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
                .to_string()
        );
        assert_eq!(
            models[1],
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string()
        );
        assert_eq!(
            models[2],
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string()
        );
        assert_eq!(
            models[3],
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string()
        );
        assert_eq!(
            models[4],
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .to_string()
        );
    }
}
