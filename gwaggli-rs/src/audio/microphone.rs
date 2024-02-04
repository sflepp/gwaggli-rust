use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, SupportedStreamConfig};

use std::error::Error;

use futures::Stream;

use futures::StreamExt;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

pub struct MicrophoneReader {
    device: cpal::Device,
    config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,
}

impl MicrophoneReader {
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

        Ok(MicrophoneReader {
            device,
            config,
            stream: None,
        })
    }

    pub fn stream(&mut self) -> Result<impl Stream<Item = Vec<f32>>, Box<dyn Error>> {
        let (tx, rx) = channel(1000);

        let stream: cpal::Stream = self.device.build_input_stream(
            &self.config.config(),
            move |data: &[f32], _: &_| {
                tx.clone().try_send(data.to_vec()).unwrap();
            },
            |err| println!("Error: {}", err),
            None,
        )?;

        self.stream = Some(stream);

        Ok(ReceiverStream::new(rx))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::audio::microphone::MicrophoneReader;
    use futures::StreamExt;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_microphone() {
        let mut microphone = MicrophoneReader::new().unwrap();

        let mut stream = microphone.stream().unwrap();

        let received = Arc::new(Mutex::new(vec![]));
        let received_clone = received.clone();

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                let mut received = received_clone.lock().unwrap();
                received.extend(item);
            }
        });

        sleep(Duration::from_millis(50)).await;

        let received_length = received.lock().unwrap().len();

        assert!(received_length > 0);
    }
}
