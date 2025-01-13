use crate::{
        functions,
        key_event_type::KeyEventType,
    };
use evdev::Device;

const TASK_ID: &str = "MONITOR";

/// Monitors and logs all events from all devices matching the given regex.
///
/// Events are logged at the INFO level with the format:
/// `{device_name}: {event}`
///
/// The task will exit if no devices are found matching the given regex.
///
/// ## Arguments
///
/// * `device_name`: A regex to match the device name against.
///
/// ## Examples
///
/// Log all events from all devices with "" (anything) in their name.
pub async fn task(device_name: &str) {
    info!("{}", TASK_ID);
    let devices = functions::get_devices_by_regex(device_name);
    if devices.len() == 0 {
        info!("{} No devices found matching: {}", TASK_ID, device_name);
        return;
    }

    for device in devices {
        tokio::spawn(monitor_events(device));
    }
}


async fn monitor_events(device: Device) {
    let device_name = device.name().unwrap_or("<unnamed>").to_string();
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        if ev.value() != KeyEventType::PRESSED { continue; }
        info!("{}: {:?}", device_name, ev.destructure()); // use just ev if you want the number instead of the code
    }
    info!("Error reading event from {}", device_name);
}