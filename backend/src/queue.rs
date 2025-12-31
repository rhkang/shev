use tokio::sync::mpsc;

use crate::db::Event;

pub type EventSender = mpsc::Sender<Event>;
pub type EventReceiver = mpsc::Receiver<Event>;

pub fn create_event_queue(buffer_size: usize) -> (EventSender, EventReceiver) {
    mpsc::channel(buffer_size)
}
