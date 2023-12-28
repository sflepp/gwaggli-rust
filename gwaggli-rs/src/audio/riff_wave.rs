
// https://tech.ebu.ch/docs/tech/tech3285.pdf
pub struct RiffWaveHeader {
    // The letters “RIFF” in ASCII form (0x52494646 big-endian form).
    chunk_id: [u8; 4], // RIFF
    // The size of the rest of the chunk following this number. This is the size of the
    // entire file in bytes minus 8 bytes for the two fields not included in this count:
    // chunk_id and chunk_size.
    chunk_size: u32,
    // The letters “WAVE” in ASCII form (0x57415645 big-endian form).
    format: [u8; 4],
    // The letters “fmt ” in ASCII form (0x666d7420 big-endian form).
    subchunk1_id: [u8; 4],
    // The size of the rest of the <fmt> subchunk following this number.
    subchunk1_size: u32,
    // The audio format. This is PCM = 1 (i.e. Linear quantization). Values other
    // than 1 indicate some form of compression.
    audio_format: AudioFormat,
    // The number of channels represented in the waveform data: 1 for mono or 2 for
    // stereo.
    num_channels: Channels,
    // The sampling rate (in sample per second) at which each channel should be
    // played. E.g. 44100, 48000 etc.
    sample_rate: u32,
    // The average number of bytes per second at which the waveform data should be
    // transferred. Playback software can estimate the buffer size using this value.
    // E.g. 176400, 192000 etc.
    byte_rate: u32,
    // The block alignment (in bytes) of the waveform data. This is the number of
    // bytes per sample including all channels. I.e. block_align = num_channels *
    // bits_per_sample/8. I.e. 2 bytes for mono and 4 bytes for stereo.
    block_align: u16,
    // The number of bits used to represent each sample of a single channel of audio.
    // I.e. 8 bits for 8-bit mono, 16 bits for 16-bit stereo etc.
    bits_per_sample: u16,
    // The letters “data” in ASCII form (0x64617461 big-endian form).
    subchunk2_id: [u8; 4],
    // The size of the data subchunk following this number.
    subchunk2_size: u32,
}

impl RiffWaveHeader {
    pub fn new(data: &[u8; 44]) -> Self {
        Self::from_bytes(data)
    }

    pub fn from_bytes(data: &[u8; 44]) -> Self {
        RiffWaveHeader {
            chunk_id: data[0..4].try_into().unwrap(),
            chunk_size: as_u32_le(data[4..8].try_into().unwrap()),
            format: data[8..12].try_into().unwrap(),
            subchunk1_id: data[12..16].try_into().unwrap(),
            subchunk1_size: as_u32_le(data[16..20].try_into().unwrap()),
            audio_format: match as_u16_le(data[20..22].try_into().unwrap()) {
                1 => AudioFormat::PCM,
                _ => panic!("Unsupported audio format"),
            },
            num_channels: match as_u16_le(data[22..24].try_into().unwrap()) {
                1 => Channels::Mono,
                2 => Channels::Stereo,
                _ => panic!("Unsupported number of channels"),
            },
            sample_rate: as_u32_le(data[24..28].try_into().unwrap()),
            byte_rate: as_u32_le(data[28..32].try_into().unwrap()),
            block_align: as_u16_le(data[32..34].try_into().unwrap()),
            bits_per_sample: as_u16_le(data[34..36].try_into().unwrap()),
            subchunk2_id: data[36..40].try_into().unwrap(),
            subchunk2_size: as_u32_le(data[40..44].try_into().unwrap()),
        }

    }
}

#[derive(PartialEq, Debug)]
enum AudioFormat {
    PCM = 1,
}

#[derive(PartialEq, Debug)]
enum Channels {
    Mono = 1,
    Stereo = 2,
}

pub struct RiffWave {
    pub header: RiffWaveHeader,
    pub data: Vec<u8>,
}

impl RiffWave {
    pub fn new(data: Vec<u8>) -> Self {
        RiffWave {
            header: RiffWaveHeader::new(data[0..44].try_into().unwrap()),
            data: data[44..].to_vec(),
        }
    }
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
        ((array[1] as u32) <<  8) +
        ((array[2] as u32) << 16) +
        ((array[3] as u32) << 24)
}

fn as_u16_le(array: &[u8; 2]) -> u16 {
    ((array[0] as u16) <<  0) +
        ((array[1] as u16) <<  8)
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use crate::audio::riff_wave::{AudioFormat, Channels, RiffWave};

    #[test]
    fn test_empty_wave_riff_header() {
        let data = b"RIFF\x24\x00\x00\x00WAVE\
    fmt \x10\x00\x00\x00\x01\x00\x01\x00\x80\x3e\x00\x00\x00\x7d\x00\x00\x02\x00\x10\x00\
    data\x00\x00\x00\x00";

        let riff_wave = RiffWave::new(data.to_vec());

        assert_eq!(riff_wave.header.chunk_id, *b"RIFF");
        assert_eq!(riff_wave.header.chunk_size, 36);
        assert_eq!(riff_wave.header.format, *b"WAVE");
        assert_eq!(riff_wave.header.subchunk1_id, *b"fmt ");
        assert_eq!(riff_wave.header.subchunk1_size, 16);
        assert_eq!(riff_wave.header.audio_format, AudioFormat::PCM);
        assert_eq!(riff_wave.header.num_channels, Channels::Mono);
        assert_eq!(riff_wave.header.sample_rate, 16000);
        assert_eq!(riff_wave.header.byte_rate, 32000);
        assert_eq!(riff_wave.header.block_align, 2);
        assert_eq!(riff_wave.header.bits_per_sample, 16);
        assert_eq!(riff_wave.header.subchunk2_id, *b"data");
        assert_eq!(riff_wave.header.subchunk2_size, 0);
        assert_eq!(riff_wave.data.len(), 0);
    }

    #[test]
    fn test_real_wave_file() {
        let file_path = "test_data/audio/riff_wave/OSR_us_000_0031_8k.wav";

        let mut file = File::open(file_path).expect("File not found");

        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data).expect("Unable to read file");

        let testee = RiffWave::new(audio_data);

        assert_eq!(testee.header.chunk_id, *b"RIFF");
        assert_eq!(testee.header.chunk_size, 674102);
        assert_eq!(testee.header.format, *b"WAVE");
        assert_eq!(testee.header.subchunk1_id, *b"fmt ");
        assert_eq!(testee.header.subchunk1_size, 16);
        assert_eq!(testee.header.audio_format, AudioFormat::PCM);
        assert_eq!(testee.header.num_channels, Channels::Mono);
        assert_eq!(testee.header.sample_rate, 8000);
        assert_eq!(testee.header.byte_rate, 16000);
        assert_eq!(testee.header.block_align, 2);
        assert_eq!(testee.header.bits_per_sample, 16);
        assert_eq!(testee.header.subchunk2_id, *b"data");
        assert_eq!(testee.header.subchunk2_size, 674066);
        assert_eq!(testee.data.len(), 674066);
    }
}
