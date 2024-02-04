use crate::audio::microphone::MicrophoneReader;
use crate::transcription::whisper::{WhisperConfig, WhisperModel, WhisperTranscriber};
use crate::transcription::{Transcribe, TranscribeRaw};
use crate::utils::sliding_window::SlidingWindow;
use futures::Stream;
use futures_util::{SinkExt, TryStreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::yield_now;
use tokio::time::sleep;
use tokio_stream::StreamExt;

pub async fn transcribe_streamed(mut microphone_reader: MicrophoneReader) {
    let mut audio_stream = microphone_reader.stream().unwrap();

    let sliding_window = Arc::new(Mutex::new(SlidingWindow::new(10 * 16_000, 4_000)));

    let sliding_window_clone = Arc::clone(&sliding_window);
    tokio::spawn(async move {
        while let Some(item) = audio_stream.next().await {
            let mut window = sliding_window_clone.lock().await;
            window.push(item);
        }
    });

    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);

    let mut transcriber = WhisperTranscriber::new(WhisperConfig {
        model: WhisperModel::Medium,
    });

    transcriber.load_context().await.unwrap();

    loop {
        let mut window = sliding_window.lock().await;
        let mut stream = Pin::new(&mut *window);

        match stream.as_mut().poll_next(&mut context) {
            Poll::Ready(Some(value)) => {
                let result = transcriber.transcribe_raw(value).unwrap();

                println!("{}", result)
            }
            Poll::Ready(None) => {
                println!("Stream ended");
                break;
            }
            Poll::Pending => {
                sleep(Duration::from_millis(1)).await;
                yield_now().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::audio::microphone::MicrophoneReader;
    use crate::transcription::real_time::transcribe_streamed;

    #[tokio::test]
    async fn test_transcribe_streamed() {
        transcribe_streamed(MicrophoneReader::new().unwrap()).await;

        println!("Done");
    }
}
