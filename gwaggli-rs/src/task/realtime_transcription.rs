use crate::event_system::event_system::EventSystem;
use crate::event_system::events::GwaggliEvent;
use crate::event_system::producer::AudioChunkProducer;

struct RealtimeTranscriptionTask {
    audio_chunk_producer: AudioChunkProducer,
}

impl RealtimeTranscriptionTask {
    pub fn new(audio_chunk_producer: AudioChunkProducer) -> Self {
        RealtimeTranscriptionTask {
            audio_chunk_producer,
        }
    }

    pub fn start(&mut self, event_system: &mut EventSystem<GwaggliEvent>) {
        self.audio_chunk_producer.produce(event_system)
    }
}
