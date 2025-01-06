use std::{thread, time::Duration};
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, Key};
use std::thread::sleep;

const TASK_ID: &str = "DEFAULT";

pub async fn task() {
    info!("{}", TASK_ID);

    //let mut keys = AttributeSet::<KeyCode>::new();
    //keys.insert(Key::BTN_DPAD_UP);

    // create a new device from an existing default
    let mut device = VirtualDeviceBuilder::new().unwrap()
        .name("Fake Keyboard")
        //.with_keys(&keys).unwrap()
        .build()
        .unwrap();

    thread::sleep(Duration::from_secs(1));


}