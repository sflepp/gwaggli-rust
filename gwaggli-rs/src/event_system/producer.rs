use crate::audio::microphone::MicrophoneAudioChunkProducer;
use crate::event_system::event_system::EventSystem;
use crate::event_system::events::GwaggliEvent;
use std::error::Error;

pub enum AudioChunkProducer {
    Microphone(MicrophoneAudioChunkProducer),
    Single(GwaggliEvent),
}

impl AudioChunkProducer {
    pub fn produce(&mut self, event_system: &mut EventSystem<GwaggliEvent>) {
        match self {
            AudioChunkProducer::Microphone(ref mut microphone_producer) => {
                microphone_producer.produce(event_system).unwrap();
            }
            AudioChunkProducer::Single(event) => {
                event_system.tx().send(event.clone()).unwrap();
            }
        }
    }
}

pub trait Producer {
    fn produce(
        &mut self,
        event_system: &mut EventSystem<GwaggliEvent>,
    ) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_system::event_system::EventSystem;
    use crate::util::{now_in_ns, recv_with_timeout};
    use AudioChunkProducer::Microphone;

    #[tokio::test]
    async fn test_microphone_audio_chunk_producer() {
        let mut testee = Microphone(MicrophoneAudioChunkProducer::new().unwrap());
        let mut event_system = EventSystem::new();
        let mut rx = event_system.rx();

        testee.produce(&mut event_system);

        let event = recv_with_timeout(&mut rx).await;

        match event {
            GwaggliEvent::AudioChunk { .. } => {}
            _ => {
                panic!("Expected GwaggliEvent::AudioChunk");
            }
        }
    }

    #[tokio::test]
    async fn test_single_audio_chunk_producer() {
        let event = GwaggliEvent::AudioChunk {
            chunk: vec![0.0; 16_000],
            sample_rate: 16_000,
            timestamp: 0,
            duration: 1_000_000_000,
        };
        let mut testee = AudioChunkProducer::Single(event);
        let mut event_system = EventSystem::new();
        let mut rx = event_system.rx();

        testee.produce(&mut event_system);

        let event = recv_with_timeout(&mut rx).await;

        match event {
            GwaggliEvent::AudioChunk {
                chunk,
                sample_rate,
                timestamp,
                duration,
            } => {
                assert_eq!(chunk, vec![0.0; 16_000]);
                assert_eq!(sample_rate, 16_000);
                assert_eq!(timestamp, 0);
                assert_eq!(duration, 1_000_000_000);
            }
            _ => {
                panic!("Expected GwaggliEvent::AudioChunk");
            }
        }
    }
}
