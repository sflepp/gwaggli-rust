use std::fmt::Display;
use std::str::from_utf8;

// https://tech.ebu.ch/docs/tech/tech3285.pdf
pub struct RiffWaveFormat {
    // The audio format. This is PCM = 1 (i.e. Linear quantization). Values other
    // than 1 indicate some form of compression.
    pub audio_format: AudioFormat,
    // The number of channels represented in the waveform data: 1 for mono or 2 for
    // stereo.
    pub num_channels: Channels,
    // The sampling rate (in sample per second) at which each channel should be
    // played. E.g. 44100, 48000 etc.
    pub sample_rate: u32,
    // The average number of bytes per second at which the waveform data should be
    // transferred. Playback software can estimate the buffer size using this value.
    // E.g. 176400, 192000 etc.
    pub byte_rate: u32,
    // The block alignment (in bytes) of the waveform data. This is the number of
    // bytes per sample including all channels. I.e. block_align = num_channels *
    // bits_per_sample/8. I.e. 2 bytes for mono and 4 bytes for stereo.
    pub block_align: u16,
    // The number of bits used to represent each sample of a single channel of audio.
    // I.e. 8 bits for 8-bit mono, 16 bits for 16-bit stereo etc.
    pub bits_per_sample: u16,
}

impl RiffWaveFormat {
    fn new(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(RiffWaveFormat {
            audio_format: match as_u16_le(bytes[0..2].try_into().unwrap()) {
                1 => AudioFormat::PCM,
                _ => panic!(
                    "Unsupported audio format: {}",
                    as_u16_le(bytes[0..2].try_into().unwrap())
                ),
            },
            num_channels: match as_u16_le(bytes[2..4].try_into().unwrap()) {
                1 => Channels::Mono,
                2 => Channels::Stereo,
                _ => panic!(
                    "Unsupported number of channels: {}",
                    as_u16_le(bytes[2..4].try_into().unwrap())
                ),
            },
            sample_rate: as_u32_le(bytes[4..8].try_into().unwrap()),
            byte_rate: as_u32_le(bytes[8..12].try_into().unwrap()),
            block_align: as_u16_le(bytes[12..14].try_into().unwrap()),
            bits_per_sample: as_u16_le(bytes[14..16].try_into().unwrap()),
        })
    }
}

#[derive(PartialEq, Debug)]
pub enum AudioFormat {
    PCM = 1,
}

impl Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::PCM => write!(f, "PCM"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Channels {
    Mono = 1,
    Stereo = 2,
}

impl Display for Channels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channels::Mono => write!(f, "Mono"),
            Channels::Stereo => write!(f, "Stereo"),
        }
    }
}

pub struct RiffWave {
    pub size: u32,
    pub format: RiffWaveFormat,
    pub data: Vec<u8>,
}

impl RiffWave {
    pub fn new(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let chunk_id: [u8; 4] = bytes[0..4].try_into().unwrap();

        match from_utf8(&chunk_id) {
            Ok("RIFF") => (),
            Ok(other) => return Err(format!("Unsupported chunk id: {}", other).into()),
            Err(e) => return Err(format!("Error parsing chunk id: {}", e).into()),
        }

        let chunk_size: u32 = as_u32_le(bytes[4..8].try_into().unwrap());
        let format: [u8; 4] = bytes[8..12].try_into().unwrap();

        match from_utf8(&format) {
            Ok("WAVE") => (),
            Ok(other) => return Err(format!("Unsupported format: {}", other).into()),
            Err(e) => return Err(format!("Error parsing format: {}", e).into()),
        }

        let mut offset = 12;

        let mut fmt: Option<RiffWaveFormat> = None;
        let mut data: Option<Vec<u8>> = None;

        loop {
            let sub_chunk_id: [u8; 4] = bytes[offset..offset + 4].try_into().unwrap();
            let sub_chunk_size: u32 = as_u32_le(bytes[offset + 4..offset + 8].try_into().unwrap());

            match from_utf8(&sub_chunk_id) {
                Ok("fmt ") => {
                    fmt = Some(RiffWaveFormat::new(
                        bytes[offset + 8..offset + 8 + sub_chunk_size as usize].to_vec(),
                    )?);
                }
                Ok("data") => {
                    data = Some(bytes[offset + 8..offset + 8 + sub_chunk_size as usize].to_vec());
                }
                Ok(_) => {
                    // skipping unknown chunk
                }
                Err(e) => return Err(format!("Error parsing sub chunk id: {}", e).into()),
            }

            offset += 8 + sub_chunk_size as usize;

            if offset >= bytes.len() {
                break;
            }
        }

        Ok(RiffWave {
            size: chunk_size,
            format: fmt.expect("Missing format"),
            data: data.expect("Missing data"),
        })
    }

    pub fn data_as_f32(&self) -> Vec<f32> {
        match self.format.audio_format {
            AudioFormat::PCM => match self.format.bits_per_sample {
                16 => return from_i16_vec_to_f32_vec(&self.data),
                _ => panic!(
                    "Unsupported bits per sample: {}",
                    self.format.bits_per_sample
                ),
            },
        }
    }
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 0)
        + ((array[1] as u32) << 8)
        + ((array[2] as u32) << 16)
        + ((array[3] as u32) << 24)
}

fn as_u16_le(array: &[u8; 2]) -> u16 {
    ((array[0] as u16) << 0) + ((array[1] as u16) << 8)
}

fn from_i16_vec_to_f32_vec(array: &[u8]) -> Vec<f32> {
    let mut result = Vec::with_capacity(array.len() / 2);

    for chunk in array.chunks_exact(2) {
        if let [low, high] = *chunk {
            let sample = i16::from_le_bytes([low, high]);
            result.push(sample as f32 * (1.0 / 32768.0));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::audio::riff_wave::{AudioFormat, Channels, RiffWave};
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_empty_wave_riff_header() {
        let data = b"RIFF\x24\x00\x00\x00WAVE\
    fmt \x10\x00\x00\x00\x01\x00\x01\x00\x80\x3e\x00\x00\x00\x7d\x00\x00\x02\x00\x10\x00\
    data\x00\x00\x00\x00";

        let riff_wave = RiffWave::new(data.to_vec()).unwrap();

        assert_eq!(riff_wave.format.audio_format, AudioFormat::PCM);
        assert_eq!(riff_wave.format.num_channels, Channels::Mono);
        assert_eq!(riff_wave.format.sample_rate, 16000);
        assert_eq!(riff_wave.format.byte_rate, 32000);
        assert_eq!(riff_wave.format.block_align, 2);
        assert_eq!(riff_wave.format.bits_per_sample, 16);
        assert_eq!(riff_wave.data.len(), 0);
    }

    #[test]
    fn test_pcm_s16le_8k_mono() {
        let data = read_audio_file("test_data/audio/riff_wave/pcm_s16le_8k_mono.wav");
        let testee = RiffWave::new(data).unwrap();

        assert_eq!(testee.format.audio_format, AudioFormat::PCM);
        assert_eq!(testee.format.num_channels, Channels::Mono);
        assert_eq!(testee.format.sample_rate, 8000);
        assert_eq!(testee.format.byte_rate, 16000);
        assert_eq!(testee.format.block_align, 2);
        assert_eq!(testee.format.bits_per_sample, 16);
        assert_eq!(testee.data.len(), 264014);
    }

    #[test]
    fn test_pcm_s16le_16k_mono() {
        let data = read_audio_file("test_data/audio/riff_wave/pcm_s16le_16k_mono.wav");
        let testee = RiffWave::new(data).unwrap();

        assert_eq!(testee.format.audio_format, AudioFormat::PCM);
        assert_eq!(testee.format.num_channels, Channels::Mono);
        assert_eq!(testee.format.sample_rate, 16000);
        assert_eq!(testee.format.byte_rate, 32000);
        assert_eq!(testee.format.block_align, 2);
        assert_eq!(testee.format.bits_per_sample, 16);
        assert_eq!(testee.data.len(), 528028);
    }

    fn read_audio_file(file_path: &str) -> Vec<u8> {
        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .expect("Unable to read file");

        audio_data
    }

    #[test]
    fn test_from_i16_vec_to_f32_vec() {
        let data = read_audio_file("test_data/audio/riff_wave/pcm_s16le_16k_mono.wav");
        let testee = RiffWave::new(data).unwrap();
        let result = super::from_i16_vec_to_f32_vec(&testee.data);

        assert_eq!(result[0], 0.0);
        assert_eq!(result[1], 6.1035156e-5);
        assert_eq!(result[1000], 0.010620117);
    }
}
