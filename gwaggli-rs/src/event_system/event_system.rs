use tokio::sync::broadcast;

pub struct EventSystem<T: Clone> {
    tx: broadcast::Sender<T>,
    rx: broadcast::Receiver<T>,
}

impl<T: Clone> EventSystem<T> {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(1000);

        EventSystem { tx, rx }
    }

    pub fn tx(&self) -> broadcast::Sender<T> {
        self.tx.clone()
    }

    pub fn rx(&self) -> broadcast::Receiver<T> {
        self.tx.subscribe()
    }
}
