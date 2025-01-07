use crate::util;
use evdev::Device;

const TASK_ID: &str = "LOG";

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