use tokio::sync::{
    mpsc,
    Mutex
};
use evdev::InputEvent;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static VIRTUAL_DEVICE_CHANNEL: Lazy<(Arc<Mutex<mpsc::Sender<InputEvent>>>, Arc<Mutex<mpsc::Receiver<InputEvent>>>)> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel::<InputEvent>(32);
    (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
});

/// Asynchronously retrieves a clone of the `Sender` for the virtual device channel.
///
/// This function acquires a lock on the transmitter part of the global virtual device channel
/// and returns a clone of it. The channel is used for sending `InputEvent` messages
/// to the virtual device. The clone will not be locked, so it can be used to send messages
/// concurrently.
///
/// ## Returns
///
/// A `Sender<InputEvent>` which can be used to send key events to the virtual device.

pub async fn get_virtual_device_tx() -> mpsc::Sender<InputEvent> {
    let tx = VIRTUAL_DEVICE_CHANNEL.0.lock().await;
    tx.clone()
}

/// Asynchronously retrieves a lock on the `Receiver` for the virtual device channel.
///
/// This function acquires a lock on the receiver part of the global virtual device channel
/// and returns a guard over it. The channel is used for receiving `InputEvent` messages
/// from the virtual device. The guard will be locked until it is dropped.
pub async fn get_virtual_device_rx() -> tokio::sync::MutexGuard<'static, mpsc::Receiver<InputEvent>> {
    VIRTUAL_DEVICE_CHANNEL.1.lock().await
}
