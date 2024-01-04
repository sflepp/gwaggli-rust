extern crate curl;

use crate::audio::riff_wave::Channels::Mono;
use crate::audio::riff_wave::RiffWave;
use crate::transcription::Transcribe;
use curl::easy::Easy;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperTranscriber {
    pub config: WhisperConfig,
    context: Option<WhisperContext>,
}

pub struct WhisperConfig {
    pub model: WhisperModel,
    pub model_dir: String,
    pub use_gpu: bool,
    pub n_threads: i32,
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

    pub fn download(&self, to_directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let model_path = to_directory.join(self.get_model_name());

        if !fs::metadata(&to_directory).is_ok() {
            fs::create_dir_all(&to_directory)?;
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
            .download(&Path::new(&self.config.model_dir))?;

        let ctx = WhisperContext::new_with_params(
            model.to_str().unwrap(),
            WhisperContextParameters {
                use_gpu: self.config.use_gpu,
            },
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
        params.set_n_threads(self.config.n_threads);
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
    use std::path::Path;

    #[test]
    fn test_transcribe_n_threads_8() {
        let file_path = "test_data/audio/riff_wave/pcm_s16le_16k_mono.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data).unwrap();

        let mut testee = WhisperTranscriber::new(WhisperConfig {
            model: WhisperModel::Tiny,
            model_dir: "./test_data/models/whisper".to_string(),
            use_gpu: true,
            n_threads: 8,
        });

        testee.load_context().unwrap();

        let result = testee.transcribe(&riff_wave).unwrap();

        assert_eq!(result, " Every word and phrase he speaks is true. He put his last cartridge into the gun and fired. They took their kids from the public school. Drive the screws straight into the wood. Keep the hatch tight and the watch constant. Sever the twine with a quick snip of the knife. Paper will dry out when wet. Drive the catch back and open the desk. Help the week to preserve their strength. A solid smile gets few friends. [BLANK_AUDIO]".to_string());
    }

    #[test]
    fn test_transcribe_n_threads_1() {
        let file_path = "test_data/audio/riff_wave/pcm_s16le_16k_mono.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data).unwrap();

        let mut testee = WhisperTranscriber::new(WhisperConfig {
            model: WhisperModel::Tiny,
            model_dir: "./test_data/models/whisper".to_string(),
            use_gpu: true,
            n_threads: 1,
        });

        testee.load_context().unwrap();

        let result = testee.transcribe(&riff_wave).unwrap();

        assert_eq!(result, " Every word and phrase he speaks is true. He put his last cartridge into the gun and fired. They took their kids from the public school. Drive the screws straight into the wood. Keep the hatch tight and the watch constant. Sever the twine with a quick snip of the knife. Paper will dry out when wet. Drive the catch back and open the desk. Help the week to preserve their strength. A solid smile gets few friends. [BLANK_AUDIO]".to_string());
    }

    #[test]
    fn test_transcribe_use_gpu_false() {
        let file_path = "test_data/audio/riff_wave/pcm_s16le_16k_mono.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data).unwrap();

        let mut testee = WhisperTranscriber::new(WhisperConfig {
            model: WhisperModel::Tiny,
            model_dir: "./test_data/models/whisper".to_string(),
            use_gpu: false,
            n_threads: 1,
        });

        testee.load_context().unwrap();

        let result = testee.transcribe(&riff_wave).unwrap();

        assert_eq!(result, " Every word and phrase he speaks is true. He put his last cartridge into the gun and fired. They took their kids from the public school. Drive the screws straight into the wood. Keep the hatch tight and the watch constant. Sever the twine with a quick snip of the knife. Paper will dry out when wet. Drive the catch back and open the desk. Help the week to preserve their strength. A solid smile gets few friends. [BLANK_AUDIO]".to_string());
    }

    #[test]
    fn test_get_model_url_tiny() {
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

    #[test]
    fn test_download_model() {
        struct Cleanup;
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all("./test_data/models/whisper/test");
            }
        }

        let _cleanup = Cleanup;

        let testee = WhisperModel::Tiny;

        testee
            .download(&Path::new("./test_data/models/whisper/test"))
            .expect("Failed to download model");
    }
}
