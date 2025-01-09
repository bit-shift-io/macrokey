use evdev::{
        KeyCode,
        KeyEvent,
    };
use crate::{
        signals,
        functions,
    };

const TASK_ID: &str = "REMAP";

/// Remaps by reading events from a given device and passing them to the virtual device.
pub async fn task() {
    info!("{}", TASK_ID);

    // predicate example
    // get by name, and not a pointer(mouse)
    let mut device = try_return!(functions::get_device_by_predicate(|d| {
        d.name().unwrap_or("") == "Lenovo ThinkPad Compact USB Keyboard with TrackPoint" && 
        !functions::get_combined_properties(d).contains(&"POINTER".to_string())
    }));

    functions::log_device_keys(&device);
    device.grab().unwrap();// lock

    // virtual device
    let tx = signals::get_virtual_device_tx().await;
    
    // monitor events
    let device_name = device.name().unwrap_or("<unnamed>").to_string();
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        info!("{}: {:?}", device_name, ev);
        // recreate event as key and pass to virtual device
        let new_event = KeyEvent::new(KeyCode::new(ev.code()), ev.value());
        tx.send(new_event).await.unwrap();
    }
}