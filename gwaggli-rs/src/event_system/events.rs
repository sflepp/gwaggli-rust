use std::fmt::Debug;

pub enum GwaggliEvent {
    AudioChunk {
        chunk: Vec<f32>,
        sample_rate: u32, // samples per second
        timestamp: u128,  // nanoseconds since UNIX_EPOCH
        duration: u128,   // nanoseconds
    },
}

impl Clone for GwaggliEvent {
    fn clone(&self) -> Self {
        match self {
            GwaggliEvent::AudioChunk {
                chunk,
                timestamp,
                duration: length,
                sample_rate,
            } => GwaggliEvent::AudioChunk {
                chunk: chunk.clone(),
                timestamp: *timestamp,
                duration: *length,
                sample_rate: *sample_rate,
            },
        }
    }
}

impl Debug for GwaggliEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GwaggliEvent::AudioChunk {
                chunk,
                timestamp,
                duration,
                sample_rate,
            } => write!(
                f,
                "GwaggliEvent::AudioChunk {{ chunk: {:?}samples, sample_rate: {} samples/s, timestamp: {}, duration: {}ns }}",
                chunk.len(), sample_rate, timestamp, duration
            ),
        }
    }
}
