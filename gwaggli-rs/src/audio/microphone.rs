use crate::event_system::event_system::EventSystem;
use crate::event_system::events::GwaggliEvent;
use crate::event_system::producer::Producer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, SupportedStreamConfig};
use std::error::Error;
use std::time::SystemTime;
use GwaggliEvent::AudioChunk;

pub struct MicrophoneAudioChunkProducer {
    device: cpal::Device,
    config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,
}

impl MicrophoneAudioChunkProducer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .expect("Failed to get default input device");

        let config = device.supported_input_configs()?
            .filter(|range| {
                range.channels() == 1 && range.sample_format() == SampleFormat::I16
            })
            .next()
            .expect("Failed to find device with following capabilities: 1 channel, 16kHz sample rate, i16 sample format");

        let config = config.with_sample_rate(SampleRate(16_000));

        Ok(MicrophoneAudioChunkProducer {
            device,
            config,
            stream: None,
        })
    }
}

impl Producer for MicrophoneAudioChunkProducer {
    fn produce(
        &mut self,
        event_system: &mut EventSystem<GwaggliEvent>,
    ) -> Result<(), Box<dyn Error>> {
        let tx = event_system.tx();
        let sample_rate = self.config.sample_rate().0;

        let stream = self.device.build_input_stream(
            &self.config.config(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let event = AudioChunk {
                    chunk: data.to_vec(),
                    sample_rate,
                    timestamp: get_timestamp(data, sample_rate),
                    duration: get_duration(data, sample_rate),
                };
                tx.send(event).unwrap();
            },
            move |err| {
                eprintln!("Error occurred on stream: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }
}

fn get_duration(data: &[f32], sample_rate: u32) -> u128 {
    (data.len() as f64 / sample_rate as f64 * 1_000_000_000.0) as u128
}

fn get_timestamp(data: &[f32], sample_rate: u32) -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        - get_duration(data, sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_system::event_system::EventSystem;
    use crate::util::{now_in_ns, recv_with_timeout};
    use std::time::Duration;

    #[tokio::test]
    async fn test_produce() {
        let mut testee = MicrophoneAudioChunkProducer::new().unwrap();
        let mut event_system = EventSystem::new();
        let mut rx = event_system.rx();

        testee.produce(&mut event_system).unwrap();

        let event = recv_with_timeout(&mut rx).await;

        match event {
            AudioChunk {
                chunk,
                sample_rate,
                timestamp,
                duration,
            } => {
                assert!(chunk.len() > 0);
                assert_eq!(sample_rate, 16_000);
                assert!(timestamp > now_in_ns() - Duration::from_millis(100).as_nanos());
                assert!(timestamp < now_in_ns());
                assert!(duration > 0);
            }
            _ => panic!("Expected GwaggliEvent::AudioChunk"),
        }
    }

    #[test]
    fn test_get_duration() {
        let data = vec![0.0; 16_000];
        let sample_rate = 16_000;

        let duration = get_duration(&data, sample_rate);

        assert_eq!(duration, 1_000_000_000);
    }

    #[test]
    fn test_get_timestamp() {
        let data = vec![0.0; 16_000];
        let sample_rate = 16_000;

        let timestamp = get_timestamp(&data, sample_rate);

        let now_ns = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        assert!(timestamp > now_ns - Duration::from_millis(1001).as_nanos());
        assert!(timestamp < now_ns);
    }
}
