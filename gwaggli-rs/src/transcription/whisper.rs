extern crate curl;

use crate::audio::riff_wave::Channels::Mono;
use crate::audio::riff_wave::RiffWave;
use crate::transcription::Transcribe;
use curl::easy::Easy;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperTranscriber {
    pub config: WhisperConfig,
}

pub struct WhisperConfig {
    pub model: WhisperModel,
    pub model_dir: String,
}

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
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}?download=true",
            self.get_model_name()
        )
    }
}

impl WhisperTranscriber {
    pub fn download_model(&self) -> Result<(), Box<dyn std::error::Error>> {
        let model_dir = PathBuf::from(self.config.model_dir.clone());
        let model_path = model_dir.join(self.config.model.get_model_name());

        if !fs::metadata(&model_dir).is_ok() {
            fs::create_dir_all(&model_dir)?;
        }

        if fs::metadata(&model_path).is_ok() {
            println!(
                "Model {} already exists, skipping download.",
                self.config.model.get_model_name()
            );
            return Ok(());
        }

        println!(
            "Downloading model {} from {}",
            self.config.model.get_model_name(),
            self.config.model.get_model_url()
        );

        let mut dest = BufWriter::new(File::create(model_path).unwrap());

        let url = self.config.model.get_model_url();

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

        Ok(())
    }
}

impl Transcribe for WhisperTranscriber {
    fn transcribe(&self, data: &RiffWave) -> String {
        format!(
            "No real transcription, but returning some data. Length={}",
            data.data.len()
        );

        self.download_model().unwrap();

        let path_to_model =
            PathBuf::from(self.config.model_dir.clone()).join(self.config.model.get_model_name());

        let ctx = WhisperContext::new_with_params(
            path_to_model.to_str().unwrap(),
            WhisperContextParameters { use_gpu: true },
        )
        .expect("Failed to create Whisper context");

        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        let mut state = ctx.create_state().expect("Failed to create Whisper state");

        /* if data.header.sample_rate != 16_000 {
            panic!("Unsupported sample rate: {}hz", data.header.sample_rate);
        } */

        if data.header.num_channels != Mono {
            panic!(
                "Unsupported number of channels: {:?}",
                data.header.num_channels.to_string()
            );
        }

        state
            .full(params, &data.data_as_f32())
            .expect("Failed to run inference");

        let num_segments = state
            .full_n_segments()
            .expect("Failed to get number of segments");

        let mut result = String::from("");

        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("Failed to get segment text");
            //let start_timestamp = state.full_get_segment_t0(i).expect("Failed to get segment start timestamp");
            //let end_timestamp = state.full_get_segment_t1(i).expect("Failed to get segment end timestamp");

            result.push_str(&segment);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::transcription::Transcribe;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_transcribe() {
        let file_path = "test_data/audio/riff_wave/OSR_us_000_0031_8k.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data);

        let testee = super::WhisperTranscriber {
            config: super::WhisperConfig {
                model: super::WhisperModel::Tiny,
                model_dir: "./test_data/models/whisper".to_string(),
            },
        };

        let result = testee.transcribe(&riff_wave);

        assert_eq!(result, " Every word and phrase he speaks is true. He puts his last cartridge into the darn infired. We took the victims from the public school. Drive the school straight into the way. Keep the head straight in the words of constant. Save the twine when it cuts you for the night. Paper will dry out when it. Slide the kids back in open the desk. Stop the week to preserve their strength. I saw him smile, get some few friends.".to_string());
    }

    #[test]
    fn test_get_model_url() {
        let testee = super::WhisperModel::Tiny;

        let result = testee.get_model_url();

        assert_eq!(result, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin?download=true");
    }

    #[test]
    fn test_download_model() {
        struct Cleanup;
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all("./test_data/models/whisper/test");
            }
        }

        let _cleanup = Cleanup;

        let testee = super::WhisperTranscriber {
            config: super::WhisperConfig {
                model: super::WhisperModel::Tiny,
                model_dir: "./test_data/models/whisper/test".to_string(),
            },
        };

        let result = testee.download_model();

        match result {
            Ok(_) => {}
            Err(e) => {
                panic!("Error downloading model: {}", e.to_string())
            }
        }
    }
}
