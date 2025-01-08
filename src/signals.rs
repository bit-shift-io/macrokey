use tokio::sync::mpsc;
use evdev::KeyEvent;
use once_cell::sync::Lazy;

pub static VIRTUAL_KEYBOARD_TX: Lazy<mpsc::Sender<KeyEvent>> = Lazy::new(|| {
    let (tx, _rx) = mpsc::channel::<KeyEvent>(32);
    tx
});

pub static VIRTUAL_KEYBOARD_RX: Lazy<mpsc::Receiver<KeyEvent>> = Lazy::new(|| {
    let (_tx, rx) = mpsc::channel::<KeyEvent>(32);
    rx
});