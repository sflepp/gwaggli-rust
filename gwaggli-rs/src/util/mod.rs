use crate::event_system::events::GwaggliEvent;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast::Receiver;
use tokio::time::timeout;

pub async fn recv_with_timeout(rx: &mut Receiver<GwaggliEvent>) -> GwaggliEvent {
    let duration = Duration::from_millis(10_000);
    let future = rx.recv();
    timeout(duration, future)
        .await
        .expect("Timeout exceeded.")
        .unwrap()
}

pub fn now_in_ns() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos()
}
