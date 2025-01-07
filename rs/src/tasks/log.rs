use crate::util;
const TASK_ID: &str = "LOG";

pub async fn task(device_name: &str) {
    info!("{}", TASK_ID);
    let device = util::get_device_by_name(device_name).unwrap(); // crash here with no device found
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap(); // crash here on disconnect
        if ev.value() != 1 { continue; }
        info!("{}: {:?}", device_name, ev);
    }
}