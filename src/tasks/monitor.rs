use crate::util;
use evdev::Device;

const TASK_ID: &str = "LOG";

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
/// Log all events from all devices with "" (anything) in their name:
/// 
pub async fn task(device_name: &str) {
    info!("{}", TASK_ID);
    let devices = util::get_devices_by_regex(device_name);
    if devices.len() == 0 {
        info!("{} No devices found matching: {}", TASK_ID, device_name);
        return;
    }

    for device in devices {
        let device_name = device.name().unwrap_or("Unknown").to_string();
        tokio::spawn(log_device_events(device, device_name));
    }
}


async fn log_device_events(device: Device, device_name: String) {
    let mut events = device.into_event_stream().unwrap();
    loop {
        match events.next_event().await {
            Ok(ev) => {
                if ev.value() != 1 {
                    continue;
                }
                info!("{}: {:?}", device_name, ev);
            }
            Err(e) => {
                info!("Error reading event from {}: {:?}", device_name, e);
                break;
            }
        }
    }
}