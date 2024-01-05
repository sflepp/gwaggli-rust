use crate::audio::riff_wave::Channels::Mono;
use crate::audio::riff_wave::RiffWave;
use crate::transcription::Transcribe;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::{PathBuf};
use clap::ValueEnum;
use url::Url;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use crate::environment::fs::models_dir;
use crate::environment::http::download;

pub struct WhisperTranscriber {
    pub config: WhisperConfig,
    context: Option<WhisperContext>,
}

pub struct WhisperConfig {
    pub model: WhisperModel,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl Display for WhisperModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            WhisperModel::Tiny => "ggml-tiny.en.bin",
            WhisperModel::Base => "ggml-base.bin",
            WhisperModel::Small => "ggml-small.bin",
            WhisperModel::Medium => "ggml-medium.bin",
            WhisperModel::Large => "ggml-large-v3.bin",
        })
    }
}

impl WhisperModel {
    pub fn get_model_url(&self) -> Url {
        Url::parse(
            &*format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", self)
        ).expect("Failed to parse URL")
    }

    pub async fn download_if_not_present(&self) -> Result<PathBuf, Box<dyn Error>> {
        let model_dir = models_dir().join("whisper");
        let model_path = model_dir.join(self.to_string());

        if fs::metadata(&model_path).is_ok() {
            return Ok(model_path);
        }

        let model_path = download(
            self.get_model_url(),
            model_path.clone(),
        ).await?;

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

    pub async fn load_context(&mut self) -> Result<&Self, Box<dyn Error>> {
        let model = self.config.model.download_if_not_present().await?;

        let ctx = WhisperContext::new_with_params(
            model.to_str().unwrap(),
            WhisperContextParameters::default(),
        )?;

        self.context = Some(ctx);

        Ok(self)
    }
}

impl Transcribe for WhisperTranscriber {
    fn transcribe(&self, data: &RiffWave) -> Result<String, Box<dyn Error>> {
        if data.format.sample_rate != 16_000 {
            return Err(format!("Unsupported sample rate: {}", data.format.sample_rate, ).into());
        }

        if data.format.num_channels != Mono {
            return Err(format!(
                "Unsupported number of channels: {}",
                data.format.num_channels,
            ).into());
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

    #[tokio::test]
    async fn test_transcribe() {
        let file_path = "test_data/audio/riff_wave/pcm_s16le_16k_mono.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        let riff_wave = crate::audio::riff_wave::RiffWave::new(audio_data).unwrap();

        let mut testee = WhisperTranscriber::new(WhisperConfig {
            model: WhisperModel::Tiny
        });

        testee.load_context().await.unwrap();

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
            .map(|m| m.get_model_url().to_string());

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
