use evdev::Key;
//use evdev_rs::util;
use crate::util;

const TASK_ID: &str = "TEST";

pub async fn task() {
    info!("{}", TASK_ID);

    let device = util::get_device_by_name("AT Translated Set 2 keyboard").unwrap();

    //let device = Device::open("/dev/input/event10").unwrap();

    // check if the device has an ENTER key
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        println!("are you prepared to ENTER the world of evdev?");
    } else {
        println!(":(");
    }

    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();
        info!("{:?}", ev);
    }
}