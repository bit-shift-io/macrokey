use evdev::{
        EventSummary,
        EventType,
        InputEvent,
        KeyCode,
        KeyEvent
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
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_Z, _) => { // z -> windows
                let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value());
                tx.send(ie).await.unwrap();
            }
            _ => { // passthrough
                tx.send(ev).await.unwrap();
            }
        }
    }
}