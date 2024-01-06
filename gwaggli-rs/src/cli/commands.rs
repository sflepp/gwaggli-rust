use crate::audio::riff_wave::RiffWave;
use crate::environment::fs::clear_cache;
use crate::transcription::whisper::{WhisperConfig, WhisperModel, WhisperTranscriber};
use crate::transcription::Transcribe;
use clap::{Parser, Subcommand, ValueEnum};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Transcribes a an audio file into text
    Transcribe {
        /// Path to the wave file
        #[arg(short, long)]
        input: PathBuf,

        /// Quality of the transcription; Higher takes more time to process
        #[arg(short, long, value_enum, default_value = "medium", ignore_case = true)]
        quality: Quality,
    },
    /// Clears local cache in the file system (f.e. downloaded models)
    ClearCache {},
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum Quality {
    Low,
    Medium,
    High,
}

impl Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quality::Low => write!(f, "low"),
            Quality::Medium => write!(f, "medium"),
            Quality::High => write!(f, "high"),
        }
    }
}

pub async fn run() -> Result<String, Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        println!("Name: {}", name);
    }

    match &cli.command {
        Some(Commands::Transcribe { input, quality }) => {
            cmd_transcribe(input.clone(), quality.clone()).await
        }
        Some(Commands::ClearCache {}) => cmd_clear_cache(),
        None => {
            println!("No command specified");
            Ok("".to_string())
        }
    }
}

async fn cmd_transcribe(input: PathBuf, quality: Quality) -> Result<String, Box<dyn Error>> {
    println!("Transcribing file with {} quality: {:?}", quality, input);

    if !input.exists() {
        return Err(format!("File does not exist: {:?}", input).into());
    }

    let mut reader = BufReader::new(File::open(input).unwrap());
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let riff_wave = RiffWave::new(buffer).unwrap();

    let mut transcriber = WhisperTranscriber::new(WhisperConfig {
        model: match quality {
            Quality::Low => WhisperModel::Tiny,
            Quality::Medium => WhisperModel::Medium,
            Quality::High => WhisperModel::Large,
        },
    });

    let result = transcriber.load_context().await?.transcribe(&riff_wave)?;

    Ok(result)
}

fn cmd_clear_cache() -> Result<String, Box<dyn Error>> {
    clear_cache();
    Ok("Cache cleared.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let cli = Cli::parse_from(&["gwaggli", "transcribe", "--input", "test.wav"]);
        assert_eq!(
            cli.command,
            Some(Commands::Transcribe {
                input: PathBuf::from("test.wav"),
                quality: Quality::Medium
            })
        );
    }

    #[test]
    fn test_transcribe() {}
}
